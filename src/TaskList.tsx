import NewTaskButtons from './NewTaskButtons';
import Task from './Task';
import './TaskList.css';
import * as api from './api';
import { Recents, Task as TaskType } from './util/task';
import { TrackedMultiTime, TrackedTime } from './util/timecard';
import { useCallback, useEffect, useRef, useState } from 'react';

const zeroRecents: Recents = {
    starred: [],
    other: [],
};

export interface TaskListProps {
    tasksTime: TrackedMultiTime;
}

function TaskList({ tasksTime }: TaskListProps) {
    const [recents, setRecents] = useState<Recents>(zeroRecents);
    const [newTaskID, setNewTaskID] = useState<number>();
    const rerender = useState(0)[1];

    const recentsCache = useRef(recents);
    recentsCache.current = recents;
    const activeIDsCache = useRef(tasksTime.ids);
    activeIDsCache.current = tasksTime.ids;
    const tasksCache = useRef(new Map<number, TaskType>());

    const saveNewTask = useCallback((task: TaskType) => {
        if (task.id !== 0) {
            tasksCache.current.set(task.id, task);
            setNewTaskID(task.id);
        }
    }, []);

    const logToTask = useCallback(async (task: TaskType, add: boolean) => {
        let newIDs = [task.id];

        if (add) {
            newIDs.push(...activeIDsCache.current);
        }

        await api.setTasks(newIDs);

        let { starred, other } = recentsCache.current;

        starred = starred.filter((id) => id !== task.id);
        other = other.filter((id) => id !== task.id);

        if (task.starred) {
            starred.push(task.id);
        } else {
            other.push(task.id);
        }

        setRecents({ starred, other });
        await api.makeTaskRecent(task);
    }, []);

    const stopLogToTask = useCallback(async (task: TaskType) => {
        await api.setTasks(activeIDsCache.current.filter((id) => id !== task.id));
    }, []);

    const putExistingTask = useCallback(async (task: TaskType) => {
        task = await api.putTask(task, false);

        tasksCache.current.set(task.id, task);
        rerender(Date.now());
    }, []);

    const putNewTask = useCallback(async (task: TaskType) => {
        task = await api.putTask(task, true);
        setNewTaskID(undefined);

        let { starred, other } = recentsCache.current;

        starred = starred.filter((id) => id !== task.id);
        other = other.filter((id) => id !== task.id);

        if (task?.starred) {
            starred.push(task.id);
        } else {
            other.push(task.id);
        }

        tasksCache.current.set(task.id, task);

        setRecents({ starred, other });
        await api.makeTaskRecent(task);
    }, []);

    const archiveTask = useCallback(async (id: number) => {
        let { starred, other } = recentsCache.current;

        starred = starred.filter((recentID) => recentID !== id);
        other = other.filter((recentID) => recentID !== id);

        tasksCache.current.delete(id);

        setRecents({starred, other});
        await api.archiveTask(id);
    }, []);

    useEffect(() => {
        (async () => {
            const recents = await api.getRecents();
            setRecents(recents);

            const tasks = await api.getTasks([
                ...recents.starred,
                ...recents.other,
            ]);

            for (const task of tasks) {
                tasksCache.current.set(task.id, task);
            }

            rerender(Date.now());
        })().catch(console.error);
    }, []);

    function timeForTask(id: number): TrackedTime | undefined {
        let since = tasksTime.since;

        if (!tasksTime.ids.includes(id)) {
            since = null;
        }

        const accumulated = tasksTime.accumulated[id];

        if (accumulated === undefined && !since) {
            return undefined;
        }

        return {
            accumulated: accumulated || 0,
            since,
            divider: tasksTime.ids.length,
        };
    }

    const ems = [];

    if (newTaskID !== undefined) {
        const task = tasksCache.current.get(newTaskID);
        ems.push(
            <Task
                key={newTaskID}
                id={newTaskID}
                task={task}
                putTask={putNewTask}
                archiveTask={archiveTask}
                isNew={true}
                active={tasksTime.ids.includes(newTaskID)}
                {...timeForTask(newTaskID)}
            />,
        );
    } else {
        ems.push(
            <NewTaskButtons
                key="new"
                saveNewTask={saveNewTask}
                logToTask={logToTask}
            />,
        );
    }

    for (let i = recents.starred.length - 1; i >= 0; i--) {
        const id = recents.starred[i];

        if (id === newTaskID) {
            continue;
        }

        const task = tasksCache.current.get(id);

        ems.push(
            <Task
                key={id}
                id={id}
                task={task}
                logToTask={logToTask}
                stopLogToTask={stopLogToTask}
                putTask={putExistingTask}
                archiveTask={archiveTask}
                active={tasksTime.ids.includes(id)}
                {...timeForTask(id)}
            />,
        );
    }

    for (let i = recents.other.length - 1; i >= 0; i--) {
        const id = recents.other[i];

        if (id === newTaskID) {
            continue;
        }

        const task = tasksCache.current.get(id);

        ems.push(
            <Task
                key={id}
                id={id}
                task={task}
                logToTask={logToTask}
                stopLogToTask={stopLogToTask}
                putTask={putExistingTask}
                archiveTask={archiveTask}
                active={tasksTime.ids.includes(id)}
                {...timeForTask(id)}
            />,
        );
    }

    return <div className="task-list">{ems}</div>;
}

export default TaskList;
