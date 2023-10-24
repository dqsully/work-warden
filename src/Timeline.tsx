import './Timeline.css';
import { msInDay, msInMinute } from './util/duration';
import { Timecard } from './util/timecard';
import { memo, useEffect, useState } from 'react';

function toPercent(n: number): string {
    return `${(n * 100).toFixed(5)}%`;
}

type RowName = 'work' | 'break' | 'lunch' | 'idleWork' | 'activeNotWork';

const colors: { [K in RowName]: string } = {
    work: '#e73',
    break: '#37e',
    lunch: '#3c3',
    idleWork: '#e37',
    activeNotWork: '#73c',
};
const rows: { [K in RowName]: number } = {
    work: 0,
    break: 1,
    lunch: 2,
    idleWork: 3,
    activeNotWork: 3,
};

interface LineProps {
    left: number;
    width: number;
    type: RowName;
}

const Line = memo(function Line({ left, width, type }: LineProps) {
    return (
        <div
            className="timeline-line"
            style={{
                left: toPercent(left),
                width: toPercent(width),
                top: `${rows[type] * 0.5}em`,
                borderColor: colors[type],
            }}
        />
    );
});

interface NowProps {
    left: number;
}

function Now({ left }: NowProps) {
    return <div className="timeline-now" style={{ left: toPercent(left) }} />;
}

export interface TimelineProps {
    timecard: Timecard;
    partial?: boolean;
}

const Timeline = memo(function Timeline({ timecard, partial }: TimelineProps) {
    const rerender = useState(0)[1];

    useEffect(() => {
        if (partial) {
            const interval = setInterval(() => {
                rerender(Date.now());
            }, msInMinute);

            return () => {
                clearInterval(interval);
            };
        }
    }, [partial]);

    const lines: React.ReactNode[] = [];

    if (timecard.events.length > 0) {
        const dayStartDate = new Date(timecard.events[0].time);
        dayStartDate.setHours(0);
        dayStartDate.setMinutes(0);
        dayStartDate.setSeconds(0);
        dayStartDate.setMilliseconds(0);

        const dayStart = dayStartDate.getTime();

        const since: { [K in RowName]: number | null } = {
            work:
                timecard.initialState.working.since === null ? null : dayStart,
            break:
                timecard.initialState.onBreak.since === null ? null : dayStart,
            lunch:
                timecard.initialState.onLunch.since === null ? null : dayStart,
            idleWork:
                timecard.initialState.activeUntil === null &&
                timecard.initialState.working.since !== null
                    ? dayStart
                    : null,
            activeNotWork:
                timecard.initialState.activeUntil !== null &&
                timecard.initialState.working.since === null
                    ? dayStart
                    : null,
        };
        let idling = timecard.initialState.activeUntil === null; // TODO: crash detection

        function start(type: RowName, time: number) {
            since[type] = since[type] || time;
        }
        function stop(type: RowName, time: number) {
            const singleSince = since[type];

            if (singleSince !== null) {
                const duration = time - singleSince;

                if (duration >= 5 * msInMinute) {
                    lines.push(
                        <Line
                            key={`${type}-${singleSince}`}
                            left={(singleSince - dayStart) / msInDay}
                            width={duration / msInDay}
                            type={type}
                        />,
                    );
                }
            }

            since[type] = null;
        }

        for (const event of timecard.events) {
            const time = event.time.getTime();

            switch (event.type) {
                case 'ClockIn': {
                    switch (event.clock) {
                        case 'Day': {
                            if (since.work === null) {
                                if (since.activeNotWork === null) {
                                    start('idleWork', time);
                                } else {
                                    stop('activeNotWork', time);
                                }
                            }

                            start('work', time);
                            break;
                        }
                        case 'Break': {
                            start('break', time);
                            start('work', time);
                            stop('lunch', time);
                            break;
                        }
                        case 'Lunch': {
                            start('lunch', time);
                            start('work', time);
                            stop('break', time);
                            break;
                        }
                    }
                    break;
                }
                case 'ClockOut': {
                    switch (event.clock) {
                        case 'Day': {
                            if (since.work !== null) {
                                if (since.idleWork === null) {
                                    start('activeNotWork', time);
                                } else {
                                    stop('idleWork', time);
                                }
                            }

                            stop('work', time);
                            stop('break', time);
                            stop('lunch', time);
                            break;
                        }
                        case 'Break': {
                            stop('break', time);
                            break;
                        }
                        case 'Lunch': {
                            stop('lunch', time);
                            break;
                        }
                    }
                    break;
                }
                case 'Active': {
                    idling = false;
                    break;
                }
                case 'Idle': {
                    idling = true;
                    break;
                }
            }

            if (since.work === null) {
                if (idling) {
                    start('activeNotWork', time);
                } else {
                    stop('activeNotWork', time);
                }
            } else {
                if (idling && since.break === null && since.lunch === null) {
                    start('idleWork', time);
                } else {
                    stop('idleWork', time);
                }
            }
        }

        if (partial) {
            const now = Date.now();

            stop('work', now);
            stop('break', now);
            stop('lunch', now);
            stop('activeNotWork', now);
            stop('idleWork', now);

            // TODO: account for daylight savings by using Date() to count msInDay instead of assuming
            lines.push(<Now key="now" left={(now - dayStart) / msInDay} />);
        }
    }

    return <div className="timeline">{lines}</div>;
});

export default Timeline;
