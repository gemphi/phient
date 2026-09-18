import React from 'react';
import { cn } from '../../../utils/cn';
import styles from './styles.module.scss';

export type StackDirection = 'row' | 'column' | 'row-reverse' | 'column-reverse';
export type StackAlign = 'start' | 'center' | 'end' | 'stretch' | 'baseline';
export type StackJustify = 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
export type StackVariant =
  | 'default'
  | 'page'
  | 'header'
  | 'footer'
  | 'sidebar'
  | 'main'
  | 'section'
  | 'content'
  | 'hero'
  | 'surface';

export type StackProps = React.HTMLAttributes<HTMLElement> & {
  as?: React.ElementType;
  direction?: StackDirection;
  gap?: number | string;
  align?: StackAlign;
  justify?: StackJustify;
  wrap?: boolean;
  fill?: boolean;
  grow?: number;
  shrink?: number;
  basis?: number | string;
  alignSelf?: 'auto' | 'start' | 'end' | 'center' | 'stretch';
  overflow?: 'visible' | 'hidden' | 'auto' | 'scroll';
  variant?: StackVariant;
  padding?: number | string;
  margin?: number | string;
  width?: number | string;
  height?: number | string;
  minWidth?: number | string;
  minHeight?: number | string;
  maxWidth?: number | string;
  maxHeight?: number | string;
  ref?: React.Ref<HTMLElement>;
};

const justifyMap: Record<StackJustify, React.CSSProperties['justifyContent']> = {
  start: 'flex-start',
  center: 'center',
  end: 'flex-end',
  between: 'space-between',
  around: 'space-around',
  evenly: 'space-evenly',
};

const alignMap: Record<StackAlign, React.CSSProperties['alignItems']> = {
  start: 'flex-start',
  center: 'center',
  end: 'flex-end',
  stretch: 'stretch',
  baseline: 'baseline',
};

const alignSelfMap: Record<NonNullable<StackProps['alignSelf']>, React.CSSProperties['alignSelf']> = {
  auto: 'auto',
  start: 'flex-start',
  end: 'flex-end',
  center: 'center',
  stretch: 'stretch',
};

export const Stack = React.forwardRef<HTMLElement, StackProps>(({
  as,
  children,
  direction = 'column',
  gap = 2,
  align = 'stretch',
  justify = 'start',
  wrap = false,
  fill = false,
  grow,
  shrink,
  basis,
  alignSelf,
  overflow,
  variant = 'default',
  padding,
  margin,
  width,
  height,
  minWidth,
  minHeight,
  maxWidth,
  maxHeight,
  className = '',
  style,
  ...props
}, ref) => {
  const Component = (as || 'div') as React.ElementType;

  const gapValue = typeof gap === 'number' ? `${gap * 0.25}rem` : gap === 'xs' ? '0.25rem' : gap === 'sm' ? '0.5rem' : gap === 'md' ? '1rem' : gap === 'lg' ? '1.5rem' : gap;
  const basisValue = typeof basis === 'number' ? `${basis}px` : basis;
  const sizeValue = (value?: number | string) => (typeof value === 'number' ? `${value}px` : value);

  const computedStyle: React.CSSProperties = {
    flexDirection: direction,
    alignItems: alignMap[align],
    justifyContent: justifyMap[justify],
    flexWrap: wrap ? 'wrap' : 'nowrap',
    gap: gapValue,
    ...(fill ? { flex: '1 1 0%' } : {}),
    ...(grow !== undefined ? { flexGrow: grow } : {}),
    ...(shrink !== undefined ? { flexShrink: shrink } : {}),
    ...(basisValue !== undefined ? { flexBasis: basisValue } : {}),
    ...(alignSelf ? { alignSelf: alignSelfMap[alignSelf] } : {}),
    ...(overflow ? { overflow } : {}),
    ...(padding !== undefined ? { padding: sizeValue(padding) } : {}),
    ...(margin !== undefined ? { margin: sizeValue(margin) } : {}),
    ...(width !== undefined ? { width: sizeValue(width) } : {}),
    ...(height !== undefined ? { height: sizeValue(height) } : {}),
    ...(minWidth !== undefined ? { minWidth: sizeValue(minWidth) } : {}),
    ...(minHeight !== undefined ? { minHeight: sizeValue(minHeight) } : {}),
    ...(maxWidth !== undefined ? { maxWidth: sizeValue(maxWidth) } : {}),
    ...(maxHeight !== undefined ? { maxHeight: sizeValue(maxHeight) } : {}),
    ...style,
  };

  return (
    <Component
      ref={ref}
      className={cn(styles.stack, variant !== 'default' && styles[`variant-${variant}`], className)}
      style={computedStyle}
      {...props}
    >
      {children}
    </Component>
  );
});

Stack.displayName = 'Stack';
