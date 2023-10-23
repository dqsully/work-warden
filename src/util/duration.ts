export interface Duration {
    secs: number;
    nanos: number;
}

export function durationToMs(duration: Duration): number {
    return duration.secs * 1000 + duration.nanos / 1_000_000;
}

export const msInSecond = 1000;
export const msInMinute = msInSecond * 60;
export const msInHour = msInMinute * 60;
export const msInDay = msInHour * 24;
export const msInWeek = msInDay * 7;

export function formatMs(ms: number): string {
    if (ms == 0) {
        return '0s';
    }

    if (ms < 0.001) {
        return `${Math.round(ms / 1_000_000)}ns`;
    } else if (ms < 1) {
        return `${Math.round(ms / 1_000)}μs`;
    } else if (ms < 1000) {
        return `${Math.round(ms)}ms`;
    }

    let remaining = ms;
    let output = '';

    if (remaining >= msInWeek) {
        const weeks = Math.floor(remaining / msInWeek);
        remaining = remaining % msInWeek;
        output += `${weeks}w`;
    }

    if (remaining >= msInDay) {
        const days = Math.floor(remaining / msInDay);
        remaining = remaining % msInDay;
        output += `${days}d`;
    }

    if (remaining >= msInHour) {
        const hours = Math.floor(remaining / msInHour);
        remaining = remaining % msInHour;
        output += `${hours}h`;
    }

    if (remaining >= msInMinute) {
        const minutes = Math.floor(remaining / msInMinute);
        remaining = remaining % msInMinute;
        output += `${minutes}m`;
    }

    if (remaining >= msInSecond) {
        const seconds = Math.floor(remaining / msInSecond);
        remaining = remaining % msInSecond;
        output += `${seconds}s`;
    }

    return output;
}
