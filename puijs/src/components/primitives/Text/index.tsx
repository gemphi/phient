import React from 'react';
import { cn } from '../../../utils/cn';
import styles from './styles.module.scss';

export type TextIntent =
  | 'default'
  | 'muted'
  | 'secondary'
  | 'primary'
  | 'success'
  | 'warning'
  | 'danger';

export type TextSize = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
export type TextWeight = 'normal' | 'medium' | 'semibold' | 'bold';
export type TextAlign = 'left' | 'center' | 'right';

export type TextProps = React.HTMLAttributes<HTMLElement> & {
  intent?: TextIntent;
  variant?: TextIntent | TextSize;
  size?: TextSize;
  weight?: TextWeight;
  align?: TextAlign;
  gradient?: boolean;
  mono?: boolean;
  preWrap?: boolean;
  blink?: boolean;
  lineHeight?: 'tight' | 'normal' | 'relaxed';
  as?: 'p' | 'span' | 'div' | 'label' | any;
  children?: React.ReactNode;
};

export const Text = ({
  intent,
  variant,
  size = 'md',
  weight = 'normal',
  align = 'left',
  gradient = false,
  mono = false,
  preWrap = false,
  blink = false,
  lineHeight = 'normal',
  as: Component = 'p',
  className = '',
  children,
  ...props
}: TextProps) => {
  const isSizeVariant = variant === 'xs' || variant === 'sm' || variant === 'md' || variant === 'lg' || variant === 'xl' || variant === '2xl';
  const resolvedSize = isSizeVariant ? variant : size;
  const resolvedIntent = intent || (!isSizeVariant ? variant : undefined) || 'default';

  return (
    <Component
      className={cn(
        styles.text,
        styles[`intent-${resolvedIntent}`],
        styles[`size-${resolvedSize}`],
        styles[`weight-${weight}`],
        styles[`align-${align}`],
        gradient && styles.gradient,
        mono && styles.mono,
        preWrap && styles.preWrap,
        blink && styles.blink,
        styles[`line-height-${lineHeight}`],
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
};
