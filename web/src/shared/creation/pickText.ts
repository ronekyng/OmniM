import type { PickMode } from '../types/creation';

type HintFn = (o: { clicks: number }) => string;

export const PICK_TEXT: Record<PickMode['mode'], HintFn[]> = {
  polyline: [
    () => '点击第 1 个点（可继续沿路径选点）',
    () => '点击下一点（双击或回车完成）',
    () => '点击下一点（双击或回车完成）',
  ],
  polygon: [
    () => '点击轮廓第 1 个点',
    () => '点击轮廓第 2 个点',
    () => '点击第 3 个点封闭轮廓（双击完成）',
    () => '点击第 N 个点（双击封闭完成）',
  ],
  segment: [
    () => '点击起点',
    () => '点击终点（完成）',
  ],
  place: [
    () => '点击放置位置',
  ],
};

export function defaultHintText(mode: PickMode): string {
  const steps = PICK_TEXT[mode.mode] ?? PICK_TEXT.place;
  return steps[steps.length - 1]({ clicks: 0 });
}