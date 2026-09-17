import React from 'react';
import { Page, Stack, Card, CardHeader, CardBody, Title, Text, Tag, Badge, Button, Icon, Callout, ProgressBar } from '@pui/components';
import { GitBranch, Play, CheckCircle, Clock } from 'lucide-react';

const PLAYBOOK_STEPS = [
  { step: 1, name: 'Ingest Radar Telemetry', agent: 'PhiOra', status: 'SUCCEEDED', duration: '14ms' },
  { step: 2, name: 'Validate Security & Cipher Signature', agent: 'PhiSec', status: 'SUCCEEDED', duration: '8ms' },
  { step: 3, name: 'Execute Quantum Phase Superposition', agent: 'PhiCal', status: 'SUCCEEDED', duration: '32ms' },
  { step: 4, name: 'Recursive Evaluation & Dispatch', agent: 'PhiMen', status: 'RUNNING', duration: '45ms' },
  { step: 5, name: 'Broadcast SSE Packet Stream', agent: 'PhiBus', status: 'PENDING', duration: '-' },
];

export const OrchestrationPage: React.FC = () => {
  return (
    <Page>
      <Stack direction="column" gap={4}>
        <Stack direction="row" justify="between" align="center">
          <Stack direction="column" gap={1}>
            <Title level={2} size="xl">AIP Logic & Playbook Orchestration Studio</Title>
            <Text size="md" intent="secondary">Monocle-grade DAG workflow execution, step-by-step telemetry, and state checkpoints.</Text>
          </Stack>
          <Button variant="primary" size="md" icon={<Icon icon={Play} size="xs" />}>
            Trigger Playbook
          </Button>
        </Stack>

        <Card variant="default" style={{ padding: '16px' }}>
          <CardHeader action={<Badge variant="primary" size="md">Workflow: SyncGlobalFleet</Badge>}>
            <Stack direction="row" align="center" gap={2}>
              <Icon icon={GitBranch} size="md" />
              <Title level={4} size="md">Playbook Execution DAG (Step 4 of 5)</Title>
            </Stack>
          </CardHeader>
          <CardBody>
            <Stack direction="column" gap={3}>
              <ProgressBar value={0.8} intent="primary" size="md" />

              <Stack direction="column" gap={2}>
                {PLAYBOOK_STEPS.map((step) => {
                  const isSuccess = step.status === 'SUCCEEDED';
                  const isRunning = step.status === 'RUNNING';
                  return (
                    <Callout
                      key={step.step}
                      intent={isSuccess ? 'primary' : isRunning ? 'primary' : 'none'}
                      title={`Step ${step.step}: ${step.name}`}
                      icon={<Icon icon={isSuccess ? CheckCircle : Clock} size="sm" />}
                    >
                      <Stack direction="row" justify="between" align="center">
                        <Stack direction="row" align="center" gap={2}>
                          <Badge variant="neutral" size="sm">Agent: {step.agent}</Badge>
                          <Tag intent={isSuccess ? 'success' : isRunning ? 'primary' : 'none'} minimal round size="sm">
                            {step.status}
                          </Tag>
                        </Stack>
                        <Text size="xs" intent="muted">Duration: {step.duration}</Text>
                      </Stack>
                    </Callout>
                  );
                })}
              </Stack>
            </Stack>
          </CardBody>
        </Card>
      </Stack>
    </Page>
  );
};
