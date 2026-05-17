/**
 * Port that the Margin Line UI talks to. A backend adapter (FastAPI) is
 * the default implementation; tests use a deterministic stub.
 */

import type {
    MarginCorrectInput,
    MarginDetectInput,
    MarginLine,
    MarginRepairInput,
} from '../domain/margin-line';

export interface MarginDetectionPort {
    detect(input: MarginDetectInput): Promise<MarginLine>;
    correct(input: MarginCorrectInput): Promise<MarginLine>;
    repair(input: MarginRepairInput): Promise<MarginLine>;
}
