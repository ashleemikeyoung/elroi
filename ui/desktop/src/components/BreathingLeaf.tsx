import { cn } from '../utils';
import { ELROI_LEAF_PATH, ELROI_LEAF_VIEWBOX } from './elroiLeaf';

interface BreathingLeafProps {
  className?: string;
}

/**
 * The brand leaf, breathing, shown wherever ElRoi is busy.
 *
 * Ported from the .run-leaf indicator in the ElRoi web app: the same mark the
 * wordmark carries rather than a generic spinner, so a wait reads as ElRoi
 * thinking rather than as chrome bolted on. The timing lives in main.css
 * (.elroi-leaf-breathe) because the reduced-motion fallback has to override it.
 */
export default function BreathingLeaf({ className = '' }: BreathingLeafProps) {
  return (
    <svg
      viewBox={ELROI_LEAF_VIEWBOX}
      aria-hidden="true"
      focusable="false"
      className={cn(
        'elroi-leaf-breathe h-4 w-auto shrink-0 text-[var(--elroi-logo-accent)]',
        className
      )}
    >
      <path fill="currentColor" d={ELROI_LEAF_PATH} />
    </svg>
  );
}
