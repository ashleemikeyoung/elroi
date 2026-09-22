import { cn } from '../utils';

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

const LEAF_PATH =
  'M361.00 557.91 C212.51 518.64 77.16 407.49 23.20 280.50 C7.86 244.41 1.06 215.37 1.02 185.80 C0.92 113.11 39.36 51.52 105.84 17.88 C137.89 1.66 174.63 -3.52 212.00 2.92 C245.53 8.70 278.96 24.36 308.53 48.15 C329.83 65.29 356.13 95.72 376.41 126.70 C440.35 224.38 467.83 352.98 452.97 485.03 C449.99 511.58 448.96 516.43 446.25 516.82 C444.65 517.05 442.90 515.91 440.13 512.82 C434.69 506.75 314.30 361.94 293.54 336.50 C284.12 324.95 271.71 309.20 265.97 301.50 C254.86 286.60 242.65 273.03 241.45 274.23 C239.73 275.93 248.46 295.27 260.56 316.62 C270.72 334.54 278.93 351.24 363.68 526.32 C372.01 543.53 377.99 557.07 377.81 558.32 C377.36 561.49 374.24 561.41 361.00 557.91 Z';

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
      className={cn(
        className,
        'relative inline-flex items-center',
        hover && 'group/with-hover'
      )}
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
          viewBox="0 0 457 562"
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
          <path fill="currentColor" d={LEAF_PATH} />
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
