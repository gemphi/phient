import React from 'react';
import { Stack, type StackProps } from '../Stack';

export type SectionProps = Omit<StackProps, 'as'> & {
  as?: 'section' | 'header' | 'footer' | 'aside' | 'main' | 'div';
};

export const Section = React.forwardRef<HTMLElement, SectionProps>(({ as = 'section', ...props }, ref) => (
  <Stack ref={ref} as={as} {...props} />
));

Section.displayName = 'Section';
