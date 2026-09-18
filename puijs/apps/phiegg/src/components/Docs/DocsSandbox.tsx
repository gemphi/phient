import React from 'react';
import { Card, CardHeader, CardBody, Title, Text, Button, Icon, Callout, Spinner, Stack } from '@pui/components';
import { Terminal, Play, CheckCircle } from 'lucide-react';

interface DocsSandboxProps {
  isRunning: boolean;
  liveResponse: Record<string, any> | null;
  onRunEndpoint: () => void;
}

export const DocsSandbox: React.FC<DocsSandboxProps> = ({ isRunning, liveResponse, onRunEndpoint }) => {
  return (
    <Card variant="default">
      <CardHeader
        action={
          <Button
            size="sm"
            variant="primary"
            onClick={onRunEndpoint}
            disabled={isRunning}
            icon={<Icon icon={Play} size="xs" />}
          >
            {isRunning ? 'Executing...' : 'Run Request'}
          </Button>
        }
      >
        <Stack direction="row" align="center" gap={2}>
          <Icon icon={Terminal} size="md" />
          <Title level={4} size="md">Live Sandbox</Title>
        </Stack>
      </CardHeader>
      <CardBody>
        <Stack direction="column" gap={2}>
          {isRunning ? (
            <Stack direction="row" align="center" justify="center" gap={2} padding={24}>
              <Spinner size="md" />
              <Text size="md">Calling endpoint...</Text>
            </Stack>
          ) : liveResponse ? (
            <Callout intent="success" title="200 OK - Execution Response" icon={<Icon icon={CheckCircle} size="sm" />}>
              <Text size="sm" mono preWrap lineHeight="relaxed">
                {JSON.stringify(liveResponse, null, 2)}
              </Text>
            </Callout>
          ) : (
            <Text size="sm" intent="muted">Click "Run Request" to execute this API endpoint against the live engine.</Text>
          )}
        </Stack>
      </CardBody>
    </Card>
  );
};
