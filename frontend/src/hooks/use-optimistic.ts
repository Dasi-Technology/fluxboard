import { useCallback, useRef } from "react";

interface OptimisticUpdate<T> {
  id: string;
  previousState: T;
  rollback: () => void;
}

export const useOptimistic = <T = unknown>() => {
  const updatesRef = useRef<Map<string, OptimisticUpdate<T>>>(new Map());

  const addUpdate = useCallback(
    (id: string, previousState: T, rollback: () => void) => {
      updatesRef.current.set(id, {
        id,
        previousState,
        rollback,
      });
    },
    []
  );

  const commitUpdate = useCallback((id: string) => {
    updatesRef.current.delete(id);
  }, []);

  const rollbackUpdate = useCallback((id: string) => {
    const update = updatesRef.current.get(id);
    if (update) {
      update.rollback();
      updatesRef.current.delete(id);
    }
  }, []);

  const rollbackAll = useCallback(() => {
    updatesRef.current.forEach((update) => {
      update.rollback();
    });
    updatesRef.current.clear();
  }, []);

  const hasUpdate = useCallback((id: string) => {
    return updatesRef.current.has(id);
  }, []);

  return {
    addUpdate,
    commitUpdate,
    rollbackUpdate,
    rollbackAll,
    hasUpdate,
  };
};
