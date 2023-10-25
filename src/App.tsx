import './App.css';
import TaskList from './TaskList';
import TimeButtons from './TimeButtons';
import TimeSummary from './TimeSummary';
import Timeline from './Timeline';
import { getCurrentTimecard } from './api';
import {
    RawTimecard,
    Timecard,
    TimecardState,
    TrackedMultiTime,
    TrackedTime,
    parseTimecard,
} from './util/timecard';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';

const zeroTrackedTime: TrackedTime = { since: null, accumulated: 0 };
const zeroTrackedMultiTime: TrackedMultiTime = { since: null, ids: [], accumulated: {} };
const zeroTimecardState: TimecardState = {
    working: zeroTrackedTime,
    onBreak: zeroTrackedTime,
    onLunch: zeroTrackedTime,
    activeUntil: null,
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
        getCurrentTimecard().then(setTimecard).catch(console.error);

        const unlistenTimecard = (async () => {
            const unlisten = await listen<RawTimecard>('timecard', (event) => {
                setTimecard(parseTimecard(event.payload));
            });

            return unlisten;
        })();

        return () => {
            unlistenTimecard.then((unlisten) => unlisten());
        };
    }, []);

    return (
        <div className="container">
            <h1>Work Warden</h1>
            <TimeButtons timecard={timecard} />
            <TimeSummary timecard={timecard} />
            <Timeline timecard={timecard} partial={true} />
            <TaskList tasksTime={timecard.currentState.tasks} />
        </div>
    );
}

export default App;
