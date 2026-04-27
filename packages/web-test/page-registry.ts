/**
 * Page Registry for Tairitsu E2E Testing
 *
 * Defines all demo pages to screenshot and test, with optional
 * interaction specs for capturing interactive states.
 */

export interface InteractionSpec {
  action: 'click' | 'hover' | 'focus';
  selector: string;
  suffix: string;
}

export interface PageSpec {
  url: string;
  name: string;
  category: string;
  description: string;
  selector?: string;
  interactions?: InteractionSpec[];
}

export const PAGES: PageSpec[] = [
  {
    url: '/',
    name: 'home',
    category: 'pages',
    description: 'Home / landing page',
  },
  {
    url: '/components/layer1/button',
    name: 'button',
    category: 'layer1',
    description: 'Button component variants',
    interactions: [
      { action: 'hover', selector: '#page-component-button .demo-block .hi-button-primary', suffix: '_hover_primary' },
    ],
  },
  {
    url: '/components/layer1/form',
    name: 'form',
    category: 'layer1',
    description: 'Form components (input, textarea, select)',
  },
  {
    url: '/components/layer1/search',
    name: 'search',
    category: 'layer1',
    description: 'Search input component',
  },
  {
    url: '/components/layer1/switch',
    name: 'switch',
    category: 'layer1',
    description: 'Switch/toggle component',
    interactions: [
      { action: 'click', selector: '.hi-switch', suffix: '_after_click' },
    ],
  },
  {
    url: '/components/layer1/feedback',
    name: 'feedback',
    category: 'layer1',
    description: 'Feedback/alert components',
  },
  {
    url: '/components/layer1/display',
    name: 'display',
    category: 'layer1',
    description: 'Display components (badge, tag)',
  },
  {
    url: '/components/layer1/avatar',
    name: 'avatar',
    category: 'layer1',
    description: 'Avatar component sizes and variants',
  },
  {
    url: '/components/layer1/image',
    name: 'image',
    category: 'layer1',
    description: 'Image component',
  },
  {
    url: '/components/layer1/tag',
    name: 'tag',
    category: 'layer1',
    description: 'Tag component variants',
    interactions: [
      { action: 'click', selector: '.hi-tag-closable .hi-tag-close-btn', suffix: '_after_close' },
    ],
  },
  {
    url: '/components/layer1/empty',
    name: 'empty',
    category: 'layer1',
    description: 'Empty state component',
  },
  {
    url: '/components/layer1/comment',
    name: 'comment',
    category: 'layer1',
    description: 'Comment component',
  },
  {
    url: '/event-test',
    name: 'event_test',
    category: 'system',
    description: 'Event bridge verification page',
    interactions: [
      { action: 'click', selector: '#event-test-btn', suffix: '_clicked' },
    ],
  },
];

export function getPageByName(name: string): PageSpec | undefined {
  return PAGES.find(p => p.name === name);
}

export function getPagesByCategory(category: string): PageSpec[] {
  return PAGES.filter(p => p.category === category);
}
