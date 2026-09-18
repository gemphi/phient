import React from 'react';
import { cn } from '../../../utils/cn';
import styles from './styles.module.scss';

export type TitleVariant = 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';

export type TitleProps = React.HTMLAttributes<HTMLHeadingElement> & {
  variant?: TitleVariant;
  level?: 1 | 2 | 3 | 4 | 5 | 6 | '1' | '2' | '3' | '4' | '5' | '6';
  size?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl';
  gradient?: boolean;
  children?: React.ReactNode;
};

export const Title = ({ variant, level, size, gradient = false, children, className = '', ...props }: TitleProps) => {
  const resolvedVariant: TitleVariant = variant || (level ? (`h${level}` as TitleVariant) : 'h1');
  const Tag = resolvedVariant;
  const sizeClass = size ? styles[`size-${size}`] : '';

  return (
    <Tag className={cn(styles.title, styles[resolvedVariant], sizeClass, gradient && styles.gradient, className)} {...props}>
      {children}
    </Tag>
  );
};
