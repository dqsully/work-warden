import { invoke } from "@tauri-apps/api";
import { ClockType, Timecard, parseTimecard } from "./util/timecard";

export async function clockIn(clock: ClockType) {
    return await invoke('clock_in', {clock});
}

export async function clockOut(clock: ClockType) {
    return await invoke('clock_out', {clock});
}

export async function getCurrentTimecard(): Promise<Timecard> {
    return parseTimecard(await invoke('get_current_timecard'));
}

export async function setTasks(tasks: number[]) {
    return await invoke('set_tasks', {tasks});
}
