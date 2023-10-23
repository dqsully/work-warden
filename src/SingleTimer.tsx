import { formatMs, msInSecond } from './util/duration';
import { TrackedTime } from './util/timecard';
import { useEffect, useState } from 'react';

export interface SingleTimerProps {
    label?: string;
    add?: TrackedTime[];
    subtract?: TrackedTime[];
}

function SingleTimer({
    label,
    add,
    subtract,
}: SingleTimerProps) {
    let totalMs = 0;
    let anySince = false;

    const now = Date.now();

    if (add !== undefined) {
        for (const { since, accumulated } of add) {
            totalMs += accumulated;

            if (since !== null) {
                anySince = true;
                totalMs += now - since.getTime();
            }
        }
    }

    if (subtract !== undefined) {
        for (const { since, accumulated } of subtract) {
            totalMs -= accumulated;

            if (since !== null) {
                anySince = true;
                totalMs -= now - since.getTime();
            }
        }
    }

    const rerender = useState(0)[1];

    useEffect(() => {
        if (anySince) {
            const interval = setInterval(() => {
                rerender(Date.now());
            }, msInSecond);

            return () => {
                clearInterval(interval);
            };
        }
    }, [anySince]);

    return (
        <div>
            {label}
            {formatMs(totalMs)}
        </div>
    );
};

export default SingleTimer;
