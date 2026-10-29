/**
 * Shape legend explaining the point markers used in scatter charts.
 *
 * Circle = pass-1 (initial) score. Diamond = pass-2 (fix) score.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

/**
 * HTML for the shape legend, displayed once per chart area
 */
export const SHAPE_LEGEND: string = h({
  tag: 'div',
  class: 'chart-legend',
  children: [
    h({
      tag: 'span',
      class: 'item',
      children: [
        h({
          tag: 'span',
          class: 'color-swatch',
          style: { '--point-color': '#666', },
        },),
        ' initial score',
      ],
    },),
    h({
      tag: 'span',
      class: 'item',
      children: [
        h({
          tag: 'span',
          class: 'color-swatch',
          attrs: { 'data-shape': 'diamond', },
          style: { '--point-color': '#666', },
        },),
        ' fix score',
      ],
    },),
  ],
},);
