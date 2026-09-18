import React from 'react';
import { cn } from '../../../utils/cn';
import { Stack } from '../../layout/Stack';
import styles from './styles.module.scss';

export const CODE_BLOCK_VARIANTS = {
  DEFAULT: 'default',
  DARK: 'dark',
  TERMINAL: 'terminal',
} as const;

export type CodeBlockVariant = (typeof CODE_BLOCK_VARIANTS)[keyof typeof CODE_BLOCK_VARIANTS];

type CodeBlockProps = React.HTMLAttributes<HTMLDivElement> & {
  variant?: CodeBlockVariant;
  header?: React.ReactNode;
  mono?: boolean;
  preWrap?: boolean;
  scrollable?: boolean;
  minHeight?: number | string;
  children?: React.ReactNode;
};

export const CodeBlock = ({
  variant = CODE_BLOCK_VARIANTS.DEFAULT,
  header,
  mono = true,
  preWrap = true,
  scrollable = false,
  minHeight,
  className = '',
  children,
  ...props
}: CodeBlockProps) => {
  const minHeightValue = typeof minHeight === 'number' ? `${minHeight}px` : minHeight;

  return (
    <Stack
      direction="column"
      className={cn(styles.wrapper, styles[variant], className)}
      minHeight={minHeightValue}
      {...props}
    >
      {header && (
        <Stack direction="row" align="center" gap={2} className={styles.header}>
          {header}
        </Stack>
      )}
      <pre
        className={cn(
          styles.codeBlock,
          mono && styles.mono,
          preWrap && styles.preWrap,
          scrollable && styles.scrollable
        )}
      >
        {children}
      </pre>
    </Stack>
  );
};

CodeBlock.displayName = 'CodeBlock';
