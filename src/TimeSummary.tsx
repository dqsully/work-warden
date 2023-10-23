import SingleTimer from './SingleTimer';
import './TimeSummary.css';
import { Timecard } from './util/timecard';

export interface TimeSummaryProps {
    timecard: Timecard;
}

function TimeSummary({ timecard }: TimeSummaryProps) {
    return (
        <div className="row time-summary">
            <SingleTimer
                label="Work - "
                add={[timecard.currentState.working]}
                subtract={[timecard.currentState.onLunch]}
            />
            <SingleTimer
                label="Break - "
                add={[timecard.currentState.onBreak]}
            />
            <SingleTimer
                label="Lunch - "
                add={[timecard.currentState.onLunch]}
            />
            <SingleTimer
                label="Idle - "
                add={[timecard.currentState.idleWork]}
            />
        </div>
    );
}

export default TimeSummary;
