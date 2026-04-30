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
