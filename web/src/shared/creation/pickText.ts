import type { PickMode } from '../types/creation';
import { t } from '../i18n';

type HintFn = (o: { clicks: number }) => string;

export const PICK_TEXT: Record<PickMode['mode'], HintFn[]> = {
  polyline: [
    () => t('pick.path.first'),
    () => t('pick.path.next'),
    () => t('pick.path.next'),
  ],
  polygon: [
    () => t('pick.poly.first'),
    () => t('pick.poly.second'),
    () => t('pick.poly.third'),
    (o) => t('pick.poly.close', { n: o.clicks + 1 }),
  ],
  segment: [
    () => t('pick.seg.start'),
    () => t('pick.seg.end'),
  ],
  place: [
    () => t('pick.place'),
  ],
};

export function defaultHintText(mode: PickMode): string {
  const steps = PICK_TEXT[mode.mode] ?? PICK_TEXT.place;
  return steps[steps.length - 1]({ clicks: 0 });
}