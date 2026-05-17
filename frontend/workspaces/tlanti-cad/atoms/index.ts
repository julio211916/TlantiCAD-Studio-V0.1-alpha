/**
 * Atomic-design atoms — minimal, single-purpose components.
 *
 * Convention: an atom has 0 dependencies on other components, only on
 * primitives (DOM elements + tailwind classes). It receives all data via
 * props.
 */

export { SliderRow } from './SliderRow';
export type { SliderRowProps } from './SliderRow';
export { CheckRow } from './CheckRow';
export type { CheckRowProps } from './CheckRow';
