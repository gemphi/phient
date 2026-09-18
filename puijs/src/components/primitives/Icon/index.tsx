import React from 'react';
import { type LucideIcon } from 'lucide-react';
import { cn } from '../../../utils/cn';
import styles from './styles.module.scss';

export const ICON_SIZES = {
  XS: 'xs',
  SM: 'sm',
  MD: 'md',
  LG: 'lg',
  XL: 'xl',
} as const;

export type IconSize = (typeof ICON_SIZES)[keyof typeof ICON_SIZES] | 'xs';

export type IconProps = {
  name?: LucideIcon;
  icon?: LucideIcon;
  size?: IconSize | number;
  tone?: 'default' | 'primary' | 'success' | 'warning' | 'danger' | 'info' | 'muted';
  className?: string;
};

const sizeMap: Record<string, number> = {
  xs: 12,
  sm: 14,
  md: 16,
  lg: 20,
  xl: 24,
};

export const Icon = ({ name, icon, size = ICON_SIZES.MD, tone = 'default', className = '' }: IconProps) => {
  const IconComponent = icon || name;
  if (!IconComponent) return null;
  const numericSize = typeof size === 'number' ? size : sizeMap[size] || 16;
  return <IconComponent className={cn(styles.icon, styles[`tone-${tone}`], className)} size={numericSize} color="currentColor" />;
};
