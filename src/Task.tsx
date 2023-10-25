import SingleTimer from './SingleTimer';
import './Task.css';
import { StoryType, Task as TaskType } from './util/task';
import { TrackedTime } from './util/timecard';
import { memo, useCallback, useMemo, useRef, useState } from 'react';

export interface TaskProps extends Partial<TrackedTime> {
    id: number;
    task?: TaskType;
    logToTask?: (task: TaskType, add: boolean) => Promise<void>;
    stopLogToTask?: (task: TaskType) => Promise<void>;
    putTask: (task: TaskType) => Promise<void>;
    archiveTask: (id: number) => Promise<void>;
    isNew?: boolean;
}

const Task = memo(
    ({
        id,
        task,
        logToTask,
        stopLogToTask,
        putTask,
        archiveTask,
        isNew,
        since,
        divider,
        accumulated,
    }: TaskProps) => {
        const [internalEditing, setEditing] = useState(false);
        const [archiveState, setArchiveState] = useState(0);
        const editing = isNew || internalEditing;
        const active = !!since;

        const inputTitleRef = useRef<HTMLInputElement>(null);
        const inputDescriptionRef = useRef<HTMLTextAreaElement>(null);
        const inputStarredRef = useRef<HTMLInputElement>(null);
        const inputStoryTypeRef = useRef<HTMLSelectElement>(null);
        const inputShortcutIDRef = useRef<HTMLInputElement>(null);

        const onSaveClick = useCallback(() => {
            const title = inputTitleRef.current?.value || task?.title || '';
            const description =
                inputDescriptionRef.current?.value || task?.description || '';
            const starred = !!inputStarredRef.current?.checked;
            const storyType =
                (inputStoryTypeRef.current?.value as StoryType) ||
                task?.storyType ||
                'feature';
            const shortcutId =
                +(inputShortcutIDRef.current?.value || '0') ||
                task?.shortcutId ||
                null;

            (async () => {
                await putTask({
                    id,
                    title,
                    description,
                    starred,
                    storyType,
                    shortcutId,
                });

                setEditing(false);
            })().catch(console.error);
        }, [id, task]);

        const onEditClick = useCallback(() => {
            console.log('edit click');
            setEditing(true);
        }, []);

        const onArchiveClick = useCallback(() => {
            if (archiveState < 2) {
                setArchiveState(archiveState + 1);
            } else {
                archiveTask(id).catch(console.error);
            }
        }, [id, archiveState]);

        const onLogClick = useCallback(
            (event: React.MouseEvent<HTMLButtonElement>) => {
                event.preventDefault();

                if (logToTask !== undefined && task !== undefined) {
                    logToTask(task, event.shiftKey).catch(console.error);
                }
            },
            [task, logToTask],
        );

        const onStopLogClick = useCallback(() => {
            if (stopLogToTask !== undefined && task !== undefined) {
                stopLogToTask(task).catch(console.error);
            }
        }, []);

        const headerEms = [];

        if (task === undefined) {
            headerEms.push(
                <span key="title" className="task-title">
                    (loading id #{id})
                </span>,
            );
        } else {
            if (editing) {
                headerEms.push(
                    <input
                        key="title"
                        type="text"
                        className="task-title"
                        ref={inputTitleRef}
                        defaultValue={task.title}
                    />,
                );
            } else {
                headerEms.push(
                    <span key="title" className="task-title">
                        {(task.starred ? '* ' : '') +
                            (task.title || '(no title)')}
                    </span>,
                );
            }
        }

        const time = useMemo(() => {
            if (since || accumulated) {
                return {
                    since: since || null,
                    accumulated: accumulated || 0,
                    divider,
                };
            }
        }, [since, divider, accumulated]);

        if (time !== undefined) {
            headerEms.push(<SingleTimer key="timer" add={[time]} />);
        }

        if (editing) {
            headerEms.push(
                <button
                    key="archive"
                    className="task-archive"
                    onClick={onArchiveClick}
                >
                    {['Archive', 'Are you sure?', 'Double sure?'][archiveState]}
                </button>,
            );

            headerEms.push(
                <button key="save" className="task-save" onClick={onSaveClick}>
                    Save
                </button>,
            );
        } else if (task !== undefined) {
            headerEms.push(
                <button key="edit" className="task-edit" onClick={onEditClick}>
                    Edit
                </button>,
            );
        }

        if (task !== undefined) {
            if (!active && logToTask) {
                headerEms.push(
                    <button key="log" className="task-log" onClick={onLogClick}>
                        Log to
                    </button>,
                );
            } else {
                headerEms.push(
                    <button
                        key="stop-log"
                        className="task-log"
                        onClick={onStopLogClick}
                    >
                        Stop log
                    </button>,
                );
            }
        }

        let editContents;

        if (editing) {
            editContents = (
                <div className="task-edit-form">
                    <textarea
                        className="task-description"
                        cols={60}
                        rows={10}
                        defaultValue={task?.description}
                        ref={inputDescriptionRef}
                    ></textarea>
                    <br />
                    <span className="edit-label">Starred: </span>
                    <input
                        type="checkbox"
                        className="task-starred"
                        defaultChecked={task?.starred}
                        ref={inputStarredRef}
                    />
                    <br />
                    <span className="edit-label">Story type: </span>
                    <select
                        className="task-story-type"
                        defaultValue={task?.storyType}
                        ref={inputStoryTypeRef}
                    >
                        <option value="feature">Feature</option>
                        <option value="bug">Bug</option>
                        <option value="chore">Chore</option>
                    </select>
                    <br />
                    <span className="edit-label">Shortcut ID: </span>
                    <input
                        type="text"
                        className="edit-shortcut-id"
                        defaultValue={task?.shortcutId || undefined}
                        ref={inputShortcutIDRef}
                    />
                </div>
            );
        }

        return (
            <div className={'task' + (active ? ' active' : '')}>
                <div className="task-header">{headerEms}</div>
                {editContents}
            </div>
        );
    },
);

export default Task;
