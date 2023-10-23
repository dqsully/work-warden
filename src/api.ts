import { invoke } from "@tauri-apps/api";
import { ClockType, Timecard, parseTimecard } from "./util/timecard";

export async function clockIn(clock: ClockType): Promise<Timecard> {
    return parseTimecard(await invoke('clock_in', {clock}));
}

export async function clockOut(clock: ClockType): Promise<Timecard> {
    return parseTimecard(await invoke('clock_out', {clock}));
}

export async function getState(): Promise<Timecard> {
    return parseTimecard(await invoke('get_state'));
}

export async function setTasks(tasks: number[]): Promise<Timecard> {
    return parseTimecard(await invoke('set_tasks', {tasks}));
}
