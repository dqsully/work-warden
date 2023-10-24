import { useCallback } from "react";
import { Timecard } from "./util/timecard";
import { clockIn, clockOut } from "./api";

export interface TimeButtonsProps {
    timecard: Timecard;
}

function TimeButtons({timecard}: TimeButtonsProps) {
    const clockInDay = useCallback(() => {
        clockIn('Day').catch(console.error);
    }, []);
    const startBreak = useCallback(() => {
        clockIn('Break').catch(console.error);
    }, []);
    const startLunch = useCallback(() => {
        clockIn('Lunch').catch(console.error);
    }, []);
    const clockOutDay = useCallback(() => {
        clockOut('Day').catch(console.error);
    }, []);
    const endBreak = useCallback(() => {
        clockOut('Break').catch(console.error);
    }, []);
    const endLunch = useCallback(() => {
        clockOut('Lunch').catch(console.error);
    }, []);

    return (
        <div className="row time-buttons">
            {timecard.currentState.working.since === null ? (
                <button onClick={clockInDay}>Clock in</button>
            ) : (
                <button onClick={clockOutDay}>Clock out</button>
            )}
            {timecard.currentState.onBreak.since === null ? (
                <button onClick={startBreak}>Start break</button>
            ) : (
                <button onClick={endBreak}>End break</button>
            )}
            {timecard.currentState.onLunch.since === null ? (
                <button onClick={startLunch}>Start lunch</button>
            ) : (
                <button onClick={endLunch}>End lunch</button>
            )}
        </div>
    )
}

export default TimeButtons;
