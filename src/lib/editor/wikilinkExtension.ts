import { autocompletion, type CompletionContext, type CompletionResult } from '@codemirror/autocomplete';
import { indexerService } from '../services/indexer';

export function createWikilinkAutocomplete(onNavigate?: (title: string) => void) {
  function wikilinkCompletionSource(context: CompletionContext): CompletionResult | null {
    // Match [[ followed by any characters except ]
    const match = context.matchBefore(/\[\[([^\]]*)$/);
    if (!match) return null;

    const query = match.text.slice(2).toLowerCase();
    const suggestions = indexerService.noteTitles.map((note) => ({
      label: note.title,
      type: 'text',
      detail: note.path,
      apply: `${note.title}]]`
    }));

    return {
      from: match.from + 2,
      options: suggestions.filter((s) => s.label.toLowerCase().includes(query)),
      validFor: /^[^\]]*$/
    };
  }

  return autocompletion({
    override: [wikilinkCompletionSource],
    defaultKeymap: true
  });
}
