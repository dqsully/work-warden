import { Duration, durationToMs } from './duration';

export type ClockType = 'Day' | 'Break' | 'Lunch';

export interface RawTrackedTime {
    since: string | null;
    accumulated: Duration;
}

export interface TrackedTime {
    since: Date | null;
    accumulated: number;
}

export interface RawTrackedMultiTime {
    since: [string, number[]] | null;
    accumulated: Record<number, Duration>;
}

export interface TrackedMultiTime {
    since: [Date, number[]] | null;
    accumulated: Record<number, number>;
}

export interface RawTimecardState {
    working: RawTrackedTime;
    onBreak: RawTrackedTime;
    onLunch: RawTrackedTime;
    idleWork: RawTrackedTime;
    isIdle: boolean;
    tasks: RawTrackedMultiTime;
}

export interface TimecardState {
    working: TrackedTime;
    onBreak: TrackedTime;
    onLunch: TrackedTime;
    idleWork: TrackedTime;
    isIdle: boolean;
    tasks: TrackedMultiTime;
}

export interface RawClockInEvent {
    type: 'ClockIn';
    time: string;
    clock: ClockType;
}

export interface RawClockOutEvent {
    type: 'ClockOut';
    time: string;
    clock: ClockType;
}

export interface RawActiveEvent {
    type: 'Active';
    time: string;
}

export interface RawIdleEvent {
    type: 'Idle';
    time: string;
}

export interface RawTasksEvent {
    type: 'Tasks';
    time: string;
    tasks: number[];
}

export type RawTimecardEvent =
    | RawClockInEvent
    | RawClockOutEvent
    | RawActiveEvent
    | RawIdleEvent
    | RawTasksEvent;

export interface ClockInEvent {
    type: 'ClockIn';
    time: Date;
    clock: ClockType;
}

export interface ClockOutEvent {
    type: 'ClockOut';
    time: Date;
    clock: ClockType;
}

export interface ActiveEvent {
    type: 'Active';
    time: Date;
}

export interface IdleEvent {
    type: 'Idle';
    time: Date;
}

export interface TasksEvent {
    type: 'Tasks';
    time: Date;
    tasks: number[];
}

export type TimecardEvent =
    | ClockInEvent
    | ClockOutEvent
    | ActiveEvent
    | IdleEvent
    | TasksEvent;

export interface RawTimecard {
    initialState: RawTimecardState;
    currentState: RawTimecardState;
    events: RawTimecardEvent[];
}

export interface Timecard {
    initialState: TimecardState;
    currentState: TimecardState;
    events: TimecardEvent[];
}

export function parseTimecard(raw: RawTimecard): Timecard {
    return {
        initialState: parseTimecardState(raw.initialState),
        currentState: parseTimecardState(raw.currentState),
        events: raw.events.map(parseTimecardEvent),
    }
}

export function parseTimecardState(raw: RawTimecardState): TimecardState {
    return {
        working: parseTrackedTime(raw.working),
        onBreak: parseTrackedTime(raw.onBreak),
        onLunch: parseTrackedTime(raw.onLunch),
        isIdle: raw.isIdle,
        idleWork: parseTrackedTime(raw.idleWork),
        tasks: parseTrackedMultiTime(raw.tasks),
    }
}

export function parseTrackedTime(raw: RawTrackedTime): TrackedTime {
    return {
        since: raw.since === null ? null : new Date(raw.since),
        accumulated: durationToMs(raw.accumulated),
    }
}

export function parseTrackedMultiTime(raw: RawTrackedMultiTime): TrackedMultiTime {
    const accumulated: Record<number, number> = {};

    for (const [id, duration] of Object.entries(raw.accumulated)) {
        accumulated[id as unknown as number] = durationToMs(duration);
    }

    return {
        since: raw.since === null ? null : [new Date(raw.since[0]), raw.since[1]],
        accumulated,
    }
}

export function parseTimecardEvent(raw: RawTimecardEvent): TimecardEvent {
    return {
        ...raw,
        time: new Date(raw.time),
    };
}
