import './NewTaskButtons.css';
import { putTask } from './api';
import { Task } from './util/task';
import { useCallback } from 'react';

interface NewTaskButtonsProps {
    saveNewTask: (task: Task) => void;
    logToTask: (task: Task, add: boolean) => Promise<void>;
}

const zeroTask: Task = {
    id: 0,
    shortcutId: null,
    title: '',
    description: '',
    storyType: 'feature',
    starred: false,
};

function NewTaskButtons({ saveNewTask, logToTask }: NewTaskButtonsProps) {
    const newTaskClick = useCallback(
        (e: React.MouseEvent<HTMLButtonElement>) => {
            e.preventDefault();

            (async () => {
                const task = await putTask(zeroTask, false);

                saveNewTask(task);
            })().catch(console.error);
        },
        [saveNewTask],
    );

    const logToNewTaskClick = useCallback(
        (e: React.MouseEvent<HTMLButtonElement>) => {
            e.preventDefault();

            (async () => {
                const task = await putTask(zeroTask, false);

                saveNewTask(task);

                await logToTask(task, e.shiftKey);
            })().catch(console.error);
        },
        [saveNewTask, logToTask],
    );

    return (
        <div className="row new-task-buttons">
            <button className="clickable" onClick={newTaskClick}>New Task</button>
            <button className="clickable" onClick={logToNewTaskClick}>Log to New Task</button>
        </div>
    );
}

export default NewTaskButtons;
