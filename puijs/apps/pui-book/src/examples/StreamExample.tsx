'use client';

import React from 'react';
import {
  Card,
  CardBody,
  Button,
  Stack,
  ProgressBar,
  Callout,
  CodeBlock,
  Text,
  Icon,
  useStream,
} from '@pui/components';
import { Play, Square } from 'lucide-react';

const STREAM_PROMPT =
  'Initializing Phient Autonomous Agent Kernel v2.4... Fetching ontology graph schema. Loading geosemantic index from Topos cluster. Running tensor constraint solvers. Synthesizing multi-modal operational response. Execution complete with 0 anomalies detected.';

export const StreamExample: React.FC = () => {
  const { tokens, progress, isStreaming, startStream, stopStream } = useStream();

  return (
    <Card>
      <CardBody>
        <Stack direction="column" gap={3}>
          <Callout intent="primary" title="Real-Time Streaming Engine (SSE & NDJSON)">
            Stream live LLM tokens and time-series telemetry directly into pure PUI presentation components with zero layout thrashing.
          </Callout>

          <Stack direction="row" gap={2}>
            <Button
              variant="primary"
              size="sm"
              icon={<Icon icon={Play} size="xs" />}
              disabled={isStreaming}
              onClick={() => startStream(STREAM_PROMPT)}
            >
              Start Token Stream
            </Button>
            <Button
              variant="secondary"
              size="sm"
              icon={<Icon icon={Square} size="xs" />}
              disabled={!isStreaming}
              onClick={stopStream}
            >
              Stop Stream
            </Button>
          </Stack>

          <ProgressBar value={progress / 100} intent="primary" animate={isStreaming} />

          <CodeBlock variant="terminal" minHeight={80}>
            {tokens ? (
              <Text size="sm" mono preWrap>
                {tokens}
                {isStreaming && <Text size="sm" mono blink>▍</Text>}
              </Text>
            ) : (
              <Text size="sm" mono intent="muted">
                Click "Start Token Stream" to observe live chunk emission...
              </Text>
            )}
          </CodeBlock>
        </Stack>
      </CardBody>
    </Card>
  );
};
