import React from 'react';
import { Moon, Sliders, Sparkles, Sun } from 'lucide-react';
import { Button } from '../ui/button';
import { useTheme } from '../../contexts/ThemeContext';
import { defineMessages, useIntl } from '../../i18n';
import type { ThemeId } from '../../theme/theme-tokens';

const i18n = defineMessages({
  theme: {
    id: 'themeSelector.theme',
    defaultMessage: 'Theme',
  },
  light: {
    id: 'themeSelector.light',
    defaultMessage: 'Light',
  },
  dark: {
    id: 'themeSelector.dark',
    defaultMessage: 'Dark',
  },
  aura: {
    id: 'themeSelector.aura',
    defaultMessage: 'Aura',
  },
  pink: {
    id: 'themeSelector.pink',
    defaultMessage: 'Pink',
  },
  grass: {
    id: 'themeSelector.grass',
    defaultMessage: 'Grass',
  },
  ocean: {
    id: 'themeSelector.ocean',
    defaultMessage: 'Ocean',
  },
  system: {
    id: 'themeSelector.system',
    defaultMessage: 'System',
  },
});

type ThemePreference = ThemeId | 'system';

const themeOptions: Array<{
  id: ThemePreference;
  label: keyof typeof i18n;
  swatch?: string;
  icon?: React.ComponentType<{ className?: string }>;
}> = [
  {
    id: 'light',
    label: 'light',
    icon: Sun,
    swatch: 'linear-gradient(135deg, #fffff7 0 50%, #fff7d1 50% 100%)',
  },
  {
    id: 'dark',
    label: 'dark',
    icon: Moon,
    swatch: 'linear-gradient(135deg, #16181a 0 50%, #8fabe0 50% 100%)',
  },
  {
    id: 'aura',
    label: 'aura',
    icon: Sparkles,
    swatch: 'linear-gradient(135deg, #15141b 0 50%, #a277ff 50% 100%)',
  },
  {
    id: 'pink',
    label: 'pink',
    swatch: 'linear-gradient(135deg, #faeef0 0 50%, #a8324f 50% 100%)',
  },
  {
    id: 'grass',
    label: 'grass',
    swatch: 'linear-gradient(135deg, #1e4620 0 50%, #fff0a5 50% 100%)',
  },
  {
    id: 'ocean',
    label: 'ocean',
    swatch: 'linear-gradient(135deg, #224fbc 0 50%, #ffffff 50% 100%)',
  },
  {
    id: 'system',
    label: 'system',
    icon: Sliders,
    swatch: 'linear-gradient(135deg, #ffffff 0 33%, #22252a 33% 66%, #8fabe0 66% 100%)',
  },
];

interface ThemeSelectorProps {
  className?: string;
  hideTitle?: boolean;
  horizontal?: boolean;
}

const ThemeSelector: React.FC<ThemeSelectorProps> = ({
  className = '',
  hideTitle = false,
  horizontal = false,
}) => {
  const intl = useIntl();
  const { userThemePreference, setUserThemePreference } = useTheme();

  return (
    <div className={`${!horizontal ? 'px-1 py-2 space-y-2' : ''} ${className}`}>
      {!hideTitle && <div className="text-xs text-text-primary px-3">{intl.formatMessage(i18n.theme)}</div>}
      <div
        className={`${horizontal ? 'flex flex-wrap' : 'grid grid-cols-2'} gap-1 ${!horizontal ? 'px-3' : ''}`}
      >
        {themeOptions.map((option) => {
          const Icon = option.icon;
          const selected = userThemePreference === option.id;
          return (
            <Button
              key={option.id}
              data-testid={`${option.id}-mode-button`}
              onClick={() => setUserThemePreference(option.id)}
              className={`flex items-center justify-center gap-1.5 p-2 rounded-md border transition-colors text-xs ${
                selected
                  ? 'bg-background-inverse text-text-inverse border-text-inverse hover:!bg-background-inverse hover:!text-text-inverse'
                  : 'border-border-primary hover:!bg-background-secondary text-text-secondary hover:text-text-primary'
              }`}
              variant="ghost"
              size="sm"
            >
              {Icon ? (
                <Icon className="h-3 w-3" />
              ) : (
                <span
                  className="h-3 w-3 rounded-full border border-border-tertiary"
                  style={{ background: option.swatch }}
                  aria-hidden="true"
                />
              )}
              <span>{intl.formatMessage(i18n[option.label])}</span>
            </Button>
          );
        })}
      </div>
    </div>
  );
};

export default ThemeSelector;
