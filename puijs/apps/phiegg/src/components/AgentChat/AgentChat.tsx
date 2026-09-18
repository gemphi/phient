import React from 'react';
import { Card, CardHeader, CardBody, CardFooter, Stack, Title, Text, Button, Input, Tag, Badge, Icon, Callout, Spinner } from '@pui/components';
import { useStore } from '@pui/store';
import { agentStore } from '../../state/agentStore';
import { Send, Bot, User, CheckCircle, Terminal } from 'lucide-react';

export const AgentChat: React.FC = () => {
  const { chatMessages, selectedAgentId, isStreaming, activePrompt, setPrompt, sendMessage } = useStore(agentStore);

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage(activePrompt);
    }
  };

  return (
    <Card variant="default">
      <CardHeader
        action={
          <Stack direction="row" align="center" gap={2}>
            <Tag intent="primary" minimal round size="md">
              Target: {selectedAgentId}
            </Tag>
            {isStreaming && <Spinner size="sm" />}
          </Stack>
        }
      >
        <Stack direction="row" align="center" gap={2}>
          <Icon icon={Terminal} size="md" />
          <Title level={3} size="md">AIP Multi-Turn Chat & Trace Console</Title>
        </Stack>
      </CardHeader>

      <CardBody>
        <Stack direction="column" gap={3} maxHeight={460} overflow="auto" padding="0 4px 0 0">
          {chatMessages.map((msg) => {
            const isAgent = msg.sender === 'agent';
            const isUser = msg.sender === 'user';
            return (
              <Callout
                key={msg.id}
                intent={isAgent ? 'primary' : isUser ? 'none' : 'warning'}
                title={isAgent ? `Agent [${msg.agentId || 'System'}]` : isUser ? 'Operator' : 'System Notice'}
                icon={<Icon icon={isAgent ? Bot : User} size="sm" />}
              >
                <Stack direction="column" gap={2}>
                  <Text size="md" lineHeight="normal">{msg.content}</Text>
                  {msg.traces && (
                    <Stack direction="column" gap={1} variant="surface" margin={6}>
                      <Text size="xs" weight="semibold" intent="secondary">Execution Trace Receipts:</Text>
                      {msg.traces.map((trace) => (
                        <Stack key={trace.step} direction="row" align="center" gap={2}>
                          <Icon icon={CheckCircle} size="xs" />
                          <Badge variant="neutral" size="sm">Step {trace.step}: {trace.toolName}</Badge>
                          <Text size="xs" intent="muted">{trace.durationMs}ms</Text>
                        </Stack>
                      ))}
                    </Stack>
                  )}
                  <Text size="xs" intent="muted">{msg.timestamp}</Text>
                </Stack>
              </Callout>
            );
          })}
        </Stack>
      </CardBody>

      <CardFooter>
        <Stack direction="row" gap={2} align="center" fill>
          <Input
            placeholder={`Ask ${selectedAgentId} or dispatch action...`}
            value={activePrompt}
            onChange={(e) => setPrompt(e.target.value)}
            onKeyDown={handleKeyDown}
            disabled={isStreaming}
          />
          <Button
            variant="primary"
            size="md"
            onClick={() => sendMessage(activePrompt)}
            disabled={!activePrompt.trim() || isStreaming}
            icon={<Icon icon={Send} size="sm" />}
          >
            Dispatch
          </Button>
        </Stack>
      </CardFooter>
    </Card>
  );
};
