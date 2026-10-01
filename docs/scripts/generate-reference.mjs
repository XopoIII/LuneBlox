// Writes the `@lune/*` reference pages from the standard libraries' typedefs.
//
// The typedefs (`crates/lune-std-*/types.d.luau`, or a `types/` directory module) are what
// `luneblox setup` gives the editor, and their doc comments are the only description of the
// libraries. Generating the pages from them means the reference cannot drift from what the editor
// shows. The pages are not committed: `npm run dev` and `npm run build` write them first.
//
// A doc comment is a `--[=[ ... ]=]` block of Markdown with moonwave-style tags: `@class`,
// `@within`, `@interface`, `@prop`, `@param`, `@return`, `@tag`, `@method`, `@function`. Types are
// not in the tags; they come from the Luau declaration that follows the block.

import { execFileSync } from 'node:child_process';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

const DOCS = join(dirname(fileURLToPath(import.meta.url)), '..');
const ROOT = join(DOCS, '..');
const CRATES = join(ROOT, 'crates');
const OUT = join(DOCS, 'src', 'content', 'docs', 'reference');
const EDIT_BASE = 'https://github.com/XopoIII/LuneBlox/edit/main/';

const KNOWN_TAGS = new Set(['class', 'within', 'interface', 'prop', 'param', 'return', 'tag', 'method', 'function']);
// These say which group a member belongs to, and the group's heading already says it.
const GROUPING_TAGS = new Set(['Method', 'Constructor']);

/** The typedef files of one library, the entry file first. */
function typedefFiles(crate) {
	const single = join(CRATES, crate, 'types.d.luau');
	if (existsSync(single)) {
		return [single];
	}
	const dir = join(CRATES, crate, 'types');
	const rest = readdirSync(dir)
		.filter((name) => name.endsWith('.luau') && name !== 'init.luau')
		.sort();
	return [join(dir, 'init.luau'), ...rest.map((name) => join(dir, name))];
}

/** Removes the indentation every non-blank line shares. */
function dedent(lines) {
	const indents = lines.filter((line) => line.trim() !== '').map((line) => line.match(/^[ \t]*/)[0].length);
	const shared = indents.length > 0 ? Math.min(...indents) : 0;
	return lines.map((line) => line.slice(shared));
}

function bracketDepth(text) {
	let depth = 0;
	for (const char of text) {
		if ('({['.includes(char)) depth += 1;
		if (')}]'.includes(char)) depth -= 1;
	}
	return depth;
}

/** The Luau declaration after a doc comment: a function's signature, a type, or a table field. */
function declarationAfter(lines, start) {
	let index = start;
	while (index < lines.length && lines[index].trim() === '') index += 1;
	if (index >= lines.length) return null;

	const first = lines[index].trim();
	const isFunction = first.startsWith('function ');
	const isType = first.startsWith('export type ') || first.startsWith('type ');
	const isField = /^[A-Za-z_][A-Za-z0-9_]*\s*:/.test(first);
	// A table that groups functions, such as `net.tcp`. Its name is all a page can show of it.
	const table = first.match(/^local\s+([A-Za-z_][A-Za-z0-9_]*)\s*=/);
	if (table) return { kind: 'table', name: table[1], text: null };
	if (!isFunction && !isType && !isField) return null;

	const taken = [];
	let depth = 0;
	for (; index < lines.length; index += 1) {
		const line = lines[index];
		taken.push(line);
		depth += bracketDepth(line);
		const next = (lines[index + 1] ?? '').trim();
		const continues = /[=|&]$/.test(line.trim()) || /^[|&]/.test(next);
		if (depth <= 0 && !continues) break;
	}
	const text = dedent(taken).join('\n').trimEnd();
	return { kind: isFunction ? 'function' : isType ? 'type' : 'field', text };
}

/** Every doc comment in a file, with its tags, its Markdown and the declaration it describes. */
function parseFile(path) {
	const lines = readFileSync(path, 'utf8').split('\n');
	const blocks = [];
	for (let index = 0; index < lines.length; index += 1) {
		if (!lines[index].trim().startsWith('--[=[')) continue;
		const open = index;
		while (index < lines.length && !lines[index].includes(']=]')) index += 1;

		const body = dedent(lines.slice(open + 1, index));
		const block = { path, line: open + 1, tags: [], params: [], returns: [], description: [] };
		let inFence = false;
		for (const line of body) {
			if (line.trimStart().startsWith('```')) inFence = !inFence;
			const tag = inFence ? null : line.match(/^@([A-Za-z]+)\s*(.*)$/);
			if (!tag) {
				block.description.push(line);
				continue;
			}
			const [, name, value] = tag;
			if (!KNOWN_TAGS.has(name)) {
				throw new Error(`${relative(ROOT, path)}:${open + 1}: unknown doc tag @${name}`);
			}
			if (name === 'param') block.params.push(value);
			else if (name === 'return') block.returns.push(value);
			else if (name === 'tag') block.tags.push(value.trim());
			else block[name] = value.trim();
		}
		block.description = block.description.join('\n').trim();
		block.declaration = declarationAfter(lines, index + 1);
		// An indented comment with no owner describes a field inside a type's body. The type's own
		// entry shows that body, comment included, so the field needs no entry of its own.
		if (!block.class && !block.within && /^[ \t]/.test(lines[open])) continue;
		blocks.push(block);
	}
	return blocks;
}

/** `name -- text`, `name - text` or `name text`, as the typedefs write parameters. */
function splitParam(value) {
	const match = value.match(/^(\S+)\s*(?:--?\s*)?(.*)$/);
	return { name: match[1], text: match[2] };
}

/** `type -- text` or plain text, as the typedefs write return values. */
function splitReturn(value) {
	const parts = value.split(/\s+--\s+/);
	return parts.length > 1 ? { type: parts[0], text: parts.slice(1).join(' -- ') } : { type: null, text: value };
}

/**
 * Doc comments write their own headings, such as "### Example usage", at whatever level suited the
 * editor's hover. On a page they would sit beside the members' headings and fill the table of
 * contents, so the library's own become sections and a member's become bold lines.
 */
function withHeadings(markdown, replace) {
	let inFence = false;
	return markdown
		.split('\n')
		.map((line) => {
			if (line.trimStart().startsWith('```')) inFence = !inFence;
			const heading = inFence ? null : line.match(/^#{1,6}\s+(.*)$/);
			return heading ? replace(heading[1]) : line;
		})
		.join('\n');
}

function memberName(block) {
	if (block.prop) return block.prop.split(/\s+/)[0];
	if (block.interface) return block.interface;
	if (block.method) return block.method;
	if (block.function) return block.function;
	if (block.declaration?.kind === 'table') return block.declaration.name;
	const text = block.declaration?.text ?? '';
	// A comment with no tags sits on a member of a nested table, and is named with that table.
	const nested = block.within ? null : text.match(/^function\s+([A-Za-z0-9_.]+)/);
	if (nested) return nested[1];
	const named =
		text.match(/^function\s+[A-Za-z0-9_.]*[.:]([A-Za-z0-9_]+)/) ??
		text.match(/^(?:export\s+)?type\s+([A-Za-z0-9_]+)/) ??
		text.match(/^([A-Za-z0-9_]+)\s*:/);
	if (!named) {
		throw new Error(`${relative(ROOT, block.path)}:${block.line}: doc comment describes no declaration`);
	}
	return named[1];
}

function memberGroup(block) {
	if (block.prop || block.declaration?.kind === 'table') return 'Properties';
	if (block.interface || block.declaration?.kind === 'type') return 'Types';
	if (block.tags.includes('Constructor')) return 'Constructors';
	const isMethod =
		block.tags.includes('Method') || block.method || /^function\s+[A-Za-z0-9_.]+:/.test(block.declaration?.text ?? '');
	return isMethod ? 'Methods' : 'Functions';
}

function signature(block) {
	if (block.prop) {
		const [name, ...type] = block.prop.split(/\s+/);
		return `${name}: ${type.join(' ')}`;
	}
	// A field's line ends with the comma that separates it from the next one.
	return block.declaration?.text?.replace(/,$/, '') ?? null;
}

function renderMember(block, level) {
	const out = [`${'#'.repeat(level)} ${memberName(block)}`, ''];
	const code = signature(block);
	if (code) out.push('```luau', code, '```', '');

	const labels = block.tags.filter((tag) => !GROUPING_TAGS.has(tag));
	if (labels.length > 0) out.push(labels.map((tag) => `\`${tag}\``).join(' '), '');
	if (block.description) out.push(withHeadings(block.description, (title) => `**${title}**`), '');

	if (block.params.length > 0) {
		out.push('**Parameters**', '');
		for (const { name, text } of block.params.map(splitParam)) {
			out.push(text ? `- \`${name}\` - ${text}` : `- \`${name}\``);
		}
		out.push('');
	}
	if (block.returns.length > 0) {
		out.push('**Returns**', '');
		for (const { type, text } of block.returns.map(splitReturn)) {
			out.push(type ? `- \`${type}\` - ${text}` : `- ${text}`);
		}
		out.push('');
	}
	return out;
}

const GROUP_ORDER = ['Properties', 'Constructors', 'Functions', 'Methods', 'Types'];

function renderMembers(members, level) {
	const out = [];
	for (const group of GROUP_ORDER) {
		const inGroup = members.filter((block) => memberGroup(block) === group);
		if (inGroup.length === 0) continue;
		out.push(`${'#'.repeat(level)} ${group}`, '');
		for (const block of inGroup) out.push(...renderMember(block, level + 1));
	}
	return out;
}

/** The date of the last commit that touched any of the files, for the page's "last updated". */
function lastCommitDate(files) {
	try {
		const date = execFileSync('git', ['log', '-1', '--format=%cI', '--', ...files], { cwd: ROOT, encoding: 'utf8' });
		return date.trim() || null;
	} catch {
		return null;
	}
}

function renderLibrary(name, crate) {
	const files = typedefFiles(crate);
	const blocks = files.flatMap(parseFile);

	const classes = blocks.filter((block) => block.class);
	const primary = classes.find((block) => block.class.toLowerCase() === name);
	if (!primary) throw new Error(`${crate}: no @class names the library ${name}`);

	const members = new Map(classes.map((block) => [block.class, []]));
	for (const block of blocks) {
		if (block.class) continue;
		// A top-level comment with no tags belongs to the library itself.
		const owner = members.get(block.within ?? primary.class);
		if (!owner) {
			throw new Error(`${relative(ROOT, block.path)}:${block.line}: doc comment is @within no class of this library`);
		}
		owner.push(block);
	}

	const summary = primary.description.split('\n')[0];
	const frontmatter = [
		'---',
		`title: ${JSON.stringify(name)}`,
		`description: ${JSON.stringify(`Reference for the @lune/${name} library. ${summary}.`.replace(/\.\.$/, '.'))}`,
		`editUrl: ${EDIT_BASE}${relative(ROOT, files[0])}`,
	];
	const updated = lastCommitDate(files);
	if (updated) frontmatter.push(`lastUpdated: ${updated}`);
	frontmatter.push('---', '');

	const out = [
		...frontmatter,
		withHeadings(primary.description, (title) => `## ${title}`),
		'',
		...renderMembers(members.get(primary.class), 2),
	];
	// A library's other classes, such as the reader a child process hands back, follow it on its page.
	for (const other of classes.filter((block) => block !== primary)) {
		out.push(`## ${other.class}`, '', withHeadings(other.description, (title) => `**${title}**`), '');
		for (const block of members.get(other.class)) out.push(...renderMember(block, 3));
	}
	return { text: `${out.join('\n').trimEnd()}\n`, count: blocks.length - classes.length };
}

mkdirSync(OUT, { recursive: true });
const crates = readdirSync(CRATES)
	.filter((crate) => crate.startsWith('lune-std-'))
	.sort();
for (const crate of crates) {
	const name = crate.slice('lune-std-'.length);
	const { text, count } = renderLibrary(name, crate);
	writeFileSync(join(OUT, `${name}.md`), text);
	console.log(`reference/${name}.md: ${count} members`);
}
