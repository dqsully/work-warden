import { Duration } from "./duration";

export interface Settings {
    workTarget: Duration;
    breakMax: Duration;
    lunchMax: Duration;
    idleMax: Duration;
}
