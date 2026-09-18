'use client';

import React, { useState, useEffect } from 'react';
import {
  Card,
  CardBody,
  Button,
  Stack,
  Text,
  Badge,
  Callout,
  CodeBlock,
  Icon,
  useService,
  TelemetryService,
} from '@pui/components';
import { Activity, Send, Terminal } from 'lucide-react';

export const ServiceExample: React.FC = () => {
  const telemetry = useService(TelemetryService);
  const [events, setEvents] = useState<Array<{ timestamp: string; event: string; payload: any }>>([]);

  useEffect(() => {
    const unsub = telemetry.on('track', (entry) => {
      setEvents((prev) => [entry, ...prev].slice(0, 5));
    });
    return unsub;
  }, [telemetry]);

  const fireEvent = (name: string) => {
    telemetry.track(name, { sessionId: 'sess-8891', user: 'agent-007' });
  };

  return (
    <Card>
      <CardBody>
        <Stack direction="column" gap={3}>
          <Callout intent="warning" title="Dependency Injection & Event Bus (Inversion of Control)">
            Services encapsulate asynchronous business rules, background jobs, and emit events without tying to React UI lifecycles.
          </Callout>

          <Stack direction="row" gap={2}>
            <Button
              variant="primary"
              size="sm"
              icon={<Icon icon={Send} size="xs" />}
              onClick={() => fireEvent('agent.plan.generated')}
            >
              Emit "agent.plan.generated"
            </Button>
            <Button
              variant="secondary"
              size="sm"
              icon={<Icon icon={Activity} size="xs" />}
              onClick={() => fireEvent('telemetry.heartbeat')}
            >
              Emit "telemetry.heartbeat"
            </Button>
          </Stack>

          <CodeBlock
            variant="dark"
            header={
              <>
                <Icon icon={Terminal} size="xs" tone="muted" />
                <Text size="sm" intent="muted">Service Event Stream ({events.length} received):</Text>
              </>
            }
          >
            {events.length === 0 ? (
              <Text as="span" size="sm" mono intent="muted">
                No events emitted yet. Click a button above.
              </Text>
            ) : (
              events.map((ev, i) => (
                <Text as="span" key={i} size="sm" mono preWrap>
                  <Text as="span" size="sm" mono intent="info">[{ev.timestamp.slice(11, 19)}]</Text>{' '}
                  <Text as="span" size="sm" mono intent="success" weight="semibold">{ev.event}</Text>{' '}
                  <Text as="span" size="sm" mono intent="muted">{JSON.stringify(ev.payload)}</Text>
                </Text>
              ))
            )}
          </CodeBlock>
        </Stack>
      </CardBody>
    </Card>
  );
};
