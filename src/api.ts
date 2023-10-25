import { invoke } from "@tauri-apps/api";
import { ClockType, Timecard, parseTimecard } from "./util/timecard";
import { Recents, Task } from "./util/task";

export async function clockIn(clock: ClockType) {
    return await invoke('clock_in', {clock});
}

export async function clockOut(clock: ClockType) {
    return await invoke('clock_out', {clock});
}

export async function setTasks(tasks: number[]) {
    return await invoke('set_tasks', {tasks});
}

export async function getCurrentTimecard(): Promise<Timecard> {
    return parseTimecard(await invoke('get_current_timecard'));
}

export async function getRecents(): Promise<Recents> {
    return await invoke('get_recents');
}

export async function getTasks(ids: number[]): Promise<Task[]> {
    return await invoke('get_tasks', {ids});
}

export async function putTask(task: Task, makeRecent: boolean): Promise<Task> {
    return await invoke('put_task', {task, makeRecent});
}

export async function archiveTask(id: number): Promise<void> {
    await invoke('archive_task', {id});
}

export async function makeTaskRecent(task: Task): Promise<void> {
    await invoke('make_task_recent', {task});
}
