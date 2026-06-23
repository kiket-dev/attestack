#!/usr/bin/env node
/**
 * Sync published markdown from docs/ into site/src/content/docs/
 * Run from repo root: node site/scripts/sync-content.mjs
 */
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '../..');
const DOCS = path.join(ROOT, 'docs');
const OUT = path.join(ROOT, 'site/src/content/docs');

const PAGES = [
  {
    out: 'getting-started/installation.md',
    src: path.join(DOCS, 'installation.md'),
    frontmatter: { title: 'Installation', description: 'Install Attestack from releases or build from source.' },
  },
  {
    out: 'getting-started/use-cases.md',
    src: path.join(DOCS, 'use-cases.md'),
    frontmatter: { title: 'Use cases', description: 'Who Attestack is for and what problems it solves.' },
  },
  {
    out: 'getting-started/scenarios.md',
    src: path.join(DOCS, 'scenarios.md'),
    frontmatter: { title: 'Scenarios', description: 'Step-by-step workflows for common evidence tasks.' },
  },
  {
    out: 'reference/cli.md',
    src: path.join(DOCS, 'cli-spec.md'),
    frontmatter: { title: 'CLI reference', description: 'Every attestack command, flag, and exit code.' },
  },
  {
    out: 'concepts/data-model.md',
    src: path.join(DOCS, 'data-model.md'),
    frontmatter: { title: 'Data model', description: 'Sessions, events, bundles, and on-disk layout.' },
  },
  {
    out: 'concepts/architecture.md',
    src: path.join(DOCS, 'architecture.md'),
    frontmatter: { title: 'Architecture', description: 'How Attestack components fit together.' },
  },
  {
    out: 'concepts/security-model.md',
    src: path.join(DOCS, 'security-model.md'),
    frontmatter: { title: 'Security model', description: 'Keys, privacy, redaction, and verification.' },
  },
  {
    out: 'integrations/ci.md',
    src: path.join(DOCS, 'ci-integration.md'),
    frontmatter: { title: 'CI integration', description: 'GitHub Actions, Dagger, Earthly, and generic CI wrappers.' },
  },
  {
    out: 'integrations/agent-setup.md',
    src: path.join(DOCS, 'agent-setup.md'),
    frontmatter: { title: 'Agent setup', description: 'Connect Cursor, Claude, Copilot, and other agents via MCP.' },
  },
  {
    out: 'integrations/agent-guide.md',
    src: path.join(DOCS, 'agent-guide.md'),
    frontmatter: { title: 'Agent guide', description: 'attestack agent commands and MCP tool reference.' },
  },
  {
    out: 'integrations/harnesses.md',
    src: path.join(DOCS, 'harness-integrations.md'),
    frontmatter: { title: 'Harness integrations', description: 'LangGraph, Inspect AI, OpenHands, Aider, and wrappers.' },
  },
  {
    out: 'integrations/distribution.md',
    src: path.join(DOCS, 'distribution.md'),
    frontmatter: { title: 'Distribution', description: 'Release channels, install paths, and packaging.' },
  },
  {
    out: 'project/product-brief.md',
    src: path.join(DOCS, 'product-brief.md'),
    frontmatter: { title: 'Product brief', description: 'Problem statement, scope, and positioning.' },
  },
  {
    out: 'project/contributing.md',
    src: path.join(DOCS, 'contributing.md'),
    frontmatter: { title: 'Contributing', description: 'How to contribute to Attestack.' },
  },
  {
    out: 'project/releasing.md',
    src: path.join(DOCS, 'releasing.md'),
    frontmatter: { title: 'Releasing', description: 'Maintainer release checklist and artifacts.' },
  },
  {
    out: 'project/testing-strategy.md',
    src: path.join(DOCS, 'testing-strategy.md'),
    frontmatter: { title: 'Testing strategy', description: 'How Attestack is tested across platforms.' },
  },
];

const LINK_REPLACEMENTS = [
  [/]\(installation\.md\)/g, '](/getting-started/installation/)'],
  [/]\(quickstart\.md\)/g, '](/getting-started/quickstart/)'],
  [/]\(use-cases\.md\)/g, '](/getting-started/use-cases/)'],
  [/]\(scenarios\.md\)/g, '](/getting-started/scenarios/)'],
  [/]\(cli-spec\.md\)/g, '](/reference/cli/)'],
  [/]\(data-model\.md\)/g, '](/concepts/data-model/)'],
  [/]\(architecture\.md\)/g, '](/concepts/architecture/)'],
  [/]\(security-model\.md\)/g, '](/concepts/security-model/)'],
  [/]\(ci-integration\.md\)/g, '](/integrations/ci/)'],
  [/]\(agent-setup\.md\)/g, '](/integrations/agent-setup/)'],
  [/]\(agent-guide\.md\)/g, '](/integrations/agent-guide/)'],
  [/]\(harness-integrations\.md\)/g, '](/integrations/harnesses/)'],
  [/]\(distribution\.md\)/g, '](/integrations/distribution/)'],
  [/]\(product-brief\.md\)/g, '](/project/product-brief/)'],
  [/]\(contributing\.md\)/g, '](/project/contributing/)'],
  [/]\(releasing\.md\)/g, '](/project/releasing/)'],
  [/]\(testing-strategy\.md\)/g, '](/project/testing-strategy/)'],
];

const ADMONITION_KINDS = {
  NOTE: 'note',
  TIP: 'tip',
  WARNING: 'caution',
  CAUTION: 'danger',
};

function toFrontmatter(meta) {
  const lines = ['---'];
  for (const [k, v] of Object.entries(meta)) {
    lines.push(`${k}: "${String(v).replace(/"/g, '\\"')}"`);
  }
  lines.push('---', '');
  return lines.join('\n');
}

function convertAdmonitions(body) {
  return body.replace(
    /^> \[!(NOTE|TIP|WARNING|CAUTION)\]\n((?:> ?.*\n?)*)/gm,
    (_, kind, quoted) => {
      const type = ADMONITION_KINDS[kind] ?? 'note';
      const content = quoted
        .split('\n')
        .map((line) => line.replace(/^> ?/, ''))
        .join('\n')
        .trim();
      return `:::${type}\n${content}\n:::\n\n`;
    },
  );
}

function stripLeadingTitle(body, title) {
  const firstLine = body.split('\n')[0] ?? '';
  if (firstLine.startsWith('# ') && firstLine.slice(2).toLowerCase() === title.toLowerCase()) {
    return body.split('\n').slice(1).join('\n').trimStart();
  }
  return body;
}

function transformBody(body, title) {
  let out = body;
  for (const [pat, rep] of LINK_REPLACEMENTS) {
    out = out.replace(pat, rep);
  }
  out = convertAdmonitions(out);
  out = stripLeadingTitle(out, title);
  out = out.replace(/<figure class="demo-gif-frame">[\s\S]*?<\/figure>/m, '');
  out = out.replace(/<ol class="steps">[\s\S]*?<\/ol>/m, (block) =>
    block
      .replace(/<\/?ol[^>]*>/g, '')
      .replace(/<\/?li>/g, '\n'),
  );
  return out.trim() + '\n';
}

for (const page of PAGES) {
  if (!fs.existsSync(page.src)) {
    console.error(`Missing source: ${page.src}`);
    process.exit(1);
  }
  const body = fs.readFileSync(page.src, 'utf8');
  const outPath = path.join(OUT, page.out);
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(
    outPath,
    toFrontmatter(page.frontmatter) + transformBody(body, page.frontmatter.title),
  );
  console.log(`synced ${page.out}`);
}

console.log('Content sync complete (quickstart.mdx is hand-maintained).');
