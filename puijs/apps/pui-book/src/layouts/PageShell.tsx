'use client';

import React from 'react';
import { Title, Text, Stack, Section, Divider } from '@pui/components';

interface PageShellProps {
  title: string;
  description?: string;
  badge?: React.ReactNode;
  children: React.ReactNode;
}

export const PageShell: React.FC<PageShellProps> = ({
  title,
  description,
  badge,
  children,
}) => {
  return (
    <Section variant="section" direction="column" gap={6}>
      <Stack as="header" direction="column" gap={3} padding="0 0 16px 0">
        <Stack direction="row" justify="between" align="center">
          <Title level={2}>{title}</Title>
          {badge}
        </Stack>
        {description && (
          <Text variant="base" color="secondary">
            {description}
          </Text>
        )}
        <Divider />
      </Stack>
      <Stack as="main" direction="column" gap={8}>
        {children}
      </Stack>
    </Section>
  );
};
