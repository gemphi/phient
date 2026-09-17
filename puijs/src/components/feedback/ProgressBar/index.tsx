'use client';

import React from 'react';
import { cn } from '../../../utils/cn';
import styles from './styles.module.scss';

export type ProgressBarIntent = 'primary' | 'success' | 'warning' | 'error' | 'none';
export type ProgressBarSize = 'sm' | 'md' | 'lg';

export interface ProgressBarProps extends React.HTMLAttributes<HTMLDivElement> {
  value?: number; // 0 to 1, or undefined for indeterminate
  intent?: ProgressBarIntent;
  size?: ProgressBarSize;
  animate?: boolean;
  striped?: boolean;
}

export const ProgressBar: React.FC<ProgressBarProps> = ({
  value,
  intent = 'primary',
  size = 'md',
  animate = false,
  striped = false,
  className,
  ...props
}) => {
  const isIndeterminate = value === undefined;
  const percentage = isIndeterminate ? 100 : Math.min(Math.max(value * 100, 0), 100);

  return (
    <div
      className={cn(
        styles.progressBar,
        styles[`intent-${intent}`],
        styles[`size-${size}`],
        isIndeterminate && styles.indeterminate,
        striped && styles.striped,
        animate && styles.animate,
        className
      )}
      role="progressbar"
      aria-valuenow={isIndeterminate ? undefined : Math.round(percentage)}
      aria-valuemin={0}
      aria-valuemax={100}
      {...props}
    >
      <div className={styles.meter} style={{ width: `${percentage}%` }} />
    </div>
  );
};

export default ProgressBar;
