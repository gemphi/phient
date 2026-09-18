'use client';

import React, { useState } from 'react';
import {
  createStore,
  useStore,
  Card,
  CardBody,
  Button,
  Stack,
  Text,
  Badge,
  Callout,
} from '@pui/components';
import { Undo2, Redo2, Plus, Minus, RotateCcw } from 'lucide-react';

interface CounterState {
  count: number;
  history: number[];
  future: number[];
  increment: () => void;
  decrement: () => void;
  reset: () => void;
  undo: () => void;
  redo: () => void;
}

const demoStore = createStore<CounterState>((set, get) => ({
  count: 42,
  history: [],
  future: [],
  increment: () => {
    const { count, history } = get();
    set({ count: count + 1, history: [...history, count], future: [] });
  },
  decrement: () => {
    const { count, history } = get();
    set({ count: count - 1, history: [...history, count], future: [] });
  },
  reset: () => {
    const { count, history } = get();
    set({ count: 0, history: [...history, count], future: [] });
  },
  undo: () => {
    const { count, history, future } = get();
    if (history.length === 0) return;
    const prev = history[history.length - 1];
    set({
      count: prev,
      history: history.slice(0, -1),
      future: [count, ...future],
    });
  },
  redo: () => {
    const { count, history, future } = get();
    if (future.length === 0) return;
    const next = future[0];
    set({
      count: next,
      history: [...history, count],
      future: future.slice(1),
    });
  },
}));

export const StoreExample: React.FC = () => {
  const { count, history, future, increment, decrement, reset, undo, redo } = useStore(demoStore);

  return (
    <Card>
      <CardBody>
      <Stack direction="column" gap={3}>
        <Callout intent="primary" title="Reactive Micro-Store (useSyncExternalStore)">
          Pure observable state container decoupled from React rendering tree with zero unnecessary re-renders.
        </Callout>

        <Stack direction="row" align="center" justify="space-between">
          <Stack direction="row" align="center" gap={3}>
            <Text size="2xl" weight="bold">{count}</Text>
            <Badge variant="primary">{history.length} snapshots</Badge>
          </Stack>

          <Stack direction="row" gap={2}>
            <Button variant="secondary" size="sm" icon={<Minus size={14} />} onClick={decrement}>
              Decrement
            </Button>
            <Button variant="primary" size="sm" icon={<Plus size={14} />} onClick={increment}>
              Increment
            </Button>
            <Button variant="ghost" size="sm" icon={<RotateCcw size={14} />} onClick={reset}>
              Reset
            </Button>
            <Button variant="secondary" size="sm" icon={<Undo2 size={14} />} disabled={history.length === 0} onClick={undo}>
              Undo
            </Button>
            <Button variant="secondary" size="sm" icon={<Redo2 size={14} />} disabled={future.length === 0} onClick={redo}>
              Redo
            </Button>
          </Stack>
        </Stack>
      </Stack>
      </CardBody>
    </Card>
  );
};
