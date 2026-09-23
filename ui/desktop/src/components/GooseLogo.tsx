import { cn } from '../utils';
import { ELROI_LEAF_PATH, ELROI_LEAF_VIEWBOX } from './elroiLeaf';

interface GooseLogoProps {
  className?: string;
  size?: 'default' | 'small' | 'hero';
  hover?: boolean;
}

/**
 * The ElRoi wordmark: the brand leaf plus "ElRoi" with no visual word break.
 * Keep this as live vector/text so it can follow the current theme, but avoid
 * inline baseline offsets; those clipped the leaf in the sidebar on some builds.
 */

// Charter first, as the web app does; Georgia is the fallback that ships everywhere.
const BRAND_SERIF = 'Charter, "Bitstream Charter", "Iowan Old Style", Georgia, serif';

const SIZES = {
  small: '14px',
  default: '24px',
  hero: '3.45rem',
} as const;

const LEAF_SIZES = {
  small: '12px',
  default: '19px',
  hero: '2.7rem',
} as const;

export default function GooseLogo({
  className = '',
  size = 'default',
  hover = true,
}: GooseLogoProps) {
  return (
    <div
      className={cn(className, 'relative inline-flex items-center', hover && 'group/with-hover')}
    >
      <span
        className={cn(
          'inline-flex items-center whitespace-nowrap text-[var(--elroi-logo-ink)]',
          'transition-opacity duration-300',
          hover && 'group-hover/with-hover:opacity-90'
        )}
        style={{
          fontFamily: BRAND_SERIF,
          fontSize: SIZES[size],
          fontWeight: 400,
          lineHeight: 1,
          letterSpacing: 0,
        }}
      >
        <svg
          viewBox={ELROI_LEAF_VIEWBOX}
          aria-hidden="true"
          focusable="false"
          className="mr-[0.28em] shrink-0 text-[var(--elroi-logo-accent)]"
          style={{
            height: LEAF_SIZES[size],
            width: 'auto',
            fill: 'currentColor',
            display: 'block',
          }}
        >
          <path fill="currentColor" d={ELROI_LEAF_PATH} />
        </svg>
        El
        <span
          className="text-[var(--elroi-logo-accent)]"
          style={{ fontVariantCaps: 'small-caps', letterSpacing: 0 }}
        >
          Roi
        </span>
        <span
          aria-hidden="true"
          className="text-[var(--elroi-logo-muted)]"
          style={{
            fontSize: '.56em',
            verticalAlign: '.61em',
            marginLeft: '.09em',
            letterSpacing: 0,
            fontVariantCaps: 'normal',
          }}
        >
          ™
        </span>
      </span>
    </div>
  );
}
