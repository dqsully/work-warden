import './App.css';
import { useEffect, useState } from 'react';
import { Timecard, TimecardState, TrackedMultiTime, TrackedTime } from './util/timecard';
import TimeButtons from './TimeButtons';
import TimeSummary from './TimeSummary';
import { getState } from './api';
import Timeline from './Timeline';

const zeroTrackedTime: TrackedTime = { since: null, accumulated: 0 };
const zeroTrackedMultiTime: TrackedMultiTime = { since: null, accumulated: {} };
const zeroTimecardState: TimecardState = {
    working: zeroTrackedTime,
    onBreak: zeroTrackedTime,
    onLunch: zeroTrackedTime,
    isIdle: false,
    idleWork: zeroTrackedTime,
    tasks: zeroTrackedMultiTime,
};
const zeroTimecard: Timecard = {
    initialState: zeroTimecardState,
    currentState: zeroTimecardState,
    events: [],
};

function App() {
    const [timecard, setTimecard] = useState<Timecard>(zeroTimecard);

    useEffect(() => {
        getState()
            .then(setTimecard)
            .catch(console.error);
    }, []);

    return (
        <div className="container">
            <h1>Work Warden</h1>
            <TimeButtons timecard={timecard} setTimecard={setTimecard} />
            <TimeSummary timecard={timecard} />
            <Timeline timecard={timecard} partial={true} />
        </div>
    );
}

export default App;
