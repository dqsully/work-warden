import { useCallback } from "react";
import { Timecard } from "./util/timecard";
import { clockIn, clockOut } from "./api";

export interface TimeButtonsProps {
    timecard: Timecard;
    setTimecard(state: Timecard): void;
}

function TimeButtons({timecard, setTimecard}: TimeButtonsProps) {
    const clockInDay = useCallback(async () => {
        setTimecard(await clockIn('Day'));
    }, []);
    const startBreak = useCallback(async () => {
        setTimecard(await clockIn('Break'));
    }, []);
    const startLunch = useCallback(async () => {
        setTimecard(await clockIn('Lunch'));
    }, []);
    const clockOutDay = useCallback(async () => {
        setTimecard(await clockOut('Day'));
    }, []);
    const endBreak = useCallback(async () => {
        setTimecard(await clockOut('Break'));
    }, []);
    const endLunch = useCallback(async () => {
        setTimecard(await clockOut('Lunch'));
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
