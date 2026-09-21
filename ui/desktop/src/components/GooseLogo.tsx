import { cn } from '../utils';
import ElRoiLogo from '../images/elroi-logo.png';

interface GooseLogoProps {
  className?: string;
  size?: 'default' | 'small';
  hover?: boolean;
}

export default function GooseLogo({
  className = '',
  size = 'default',
  hover = true,
}: GooseLogoProps) {
  const sizes = {
    default: {
      frame: 'w-28 h-10',
      logo: 'w-28 h-10',
    },
    small: {
      frame: 'w-16 h-6',
      logo: 'w-16 h-6',
    },
  } as const;

  const currentSize = sizes[size];

  return (
    <div
      className={cn(
        className,
        currentSize.frame,
        'relative overflow-hidden flex items-center justify-center',
        hover && 'group/with-hover'
      )}
    >
      <img
        src={ElRoiLogo}
        alt="ElRoi"
        className={cn(
          currentSize.logo,
          'object-contain transition-opacity duration-300',
          hover && 'group-hover/with-hover:opacity-90'
        )}
      />
    </div>
  );
}
