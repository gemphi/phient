import React from 'react';
import { Card, CardHeader, CardBody, CardFooter, Stack, Title, Text, Tag, Badge, Button, Icon, ProgressBar } from '@pui/components';
import { AgentMetric } from '../../types/agents';
import { Bot, Clock, Zap, Cpu, Play, CheckCircle, AlertTriangle } from 'lucide-react';

interface AgentCardProps {
  agent: AgentMetric;
  isSelected: boolean;
  onSelect: (id: string) => void;
}

export const AgentCard: React.FC<AgentCardProps> = ({ agent, isSelected, onSelect }) => {
  const getStatusIcon = () => {
    if (agent.status === 'RUNNING') return Play;
    if (agent.status === 'HEALTHY') return CheckCircle;
    return AlertTriangle;
  };

  const getIntent = () => {
    if (agent.status === 'RUNNING') return 'primary';
    if (agent.status === 'HEALTHY') return 'success';
    if (agent.status === 'DEGRADED') return 'warning';
    return 'none';
  };

  return (
    <Card
      variant={isSelected ? 'hero' : 'default'}
      selected={isSelected}
      hoverable
      onClick={() => onSelect(agent.id)}
      style={{ padding: '16px' }}
    >
      <CardHeader
        action={
          <Tag intent={getIntent()} minimal round size="md">
            <Icon icon={getStatusIcon()} size="xs" />
            <Text size="sm" weight="semibold" style={{ marginLeft: 4 }}>{agent.status}</Text>
          </Tag>
        }
      >
        <Stack direction="row" align="center" gap={2}>
          <Icon icon={Bot} size="md" />
          <Title level={4} size="md">{agent.name}</Title>
          <Badge variant="primary" size="sm">{agent.tag}</Badge>
        </Stack>
      </CardHeader>

      <CardBody>
        <Stack direction="column" gap={2}>
          <Text size="sm" intent="secondary" style={{ minHeight: '38px', lineHeight: '1.5' }}>
            {agent.description}
          </Text>

          <Stack direction="row" justify="between" align="center">
            <Stack direction="row" align="center" gap={1}>
              <Icon icon={Clock} size="sm" />
              <Text size="sm" weight="medium">{agent.latencyMs} ms</Text>
            </Stack>
            <Stack direction="row" align="center" gap={1}>
              <Icon icon={Zap} size="sm" />
              <Text size="sm" weight="medium">r = {agent.orderParam.toFixed(3)}</Text>
            </Stack>
            <Stack direction="row" align="center" gap={1}>
              <Icon icon={Cpu} size="sm" />
              <Text size="sm" weight="medium">{agent.memoryMb} MB</Text>
            </Stack>
          </Stack>

          <ProgressBar value={agent.orderParam} intent={agent.orderParam > 0.98 ? 'success' : 'primary'} size="sm" />
        </Stack>
      </CardBody>

      <CardFooter>
        <Stack direction="row" justify="between" align="center" style={{ width: '100%', marginTop: '6px' }}>
          <Text size="sm" intent="muted">Uptime: {agent.uptime}</Text>
          <Button
            size="sm"
            variant={isSelected ? 'primary' : 'ghost'}
            onClick={(e) => {
              e.stopPropagation();
              onSelect(agent.id);
            }}
          >
            {isSelected ? 'Active Agent' : 'Dispatch'}
          </Button>
        </Stack>
      </CardFooter>
    </Card>
  );
};
