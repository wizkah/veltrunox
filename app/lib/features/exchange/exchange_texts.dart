import '../../shared_ui/app_language.dart';
import 'exchange_models.dart';

/// User-visible texts of the bookmark exchange.
abstract final class ExchangeTexts {
  static String get menu => tr('Export und Import', 'Export and import');
  static String get exportJson => tr('Exportieren als JSON', 'Export as JSON');
  static String get exportCsv => tr('Exportieren als CSV', 'Export as CSV');
  static String get import => tr('Importieren …', 'Import …');

  static String exported(String fileName) =>
      tr('Gespeichert: $fileName', 'Saved: $fileName');
  static String exportFailed(String reason) => tr(
    'Der Export ist fehlgeschlagen. $reason',
    'The export failed. $reason',
  );

  static String get previewTitle => tr('Import-Vorschau', 'Import preview');
  static String get close => tr('Schließen', 'Close');
  static String get cancel => tr('Abbrechen', 'Cancel');
  static String get apply => tr('Anwenden', 'Apply');
  static String get reading =>
      tr('Die Datei wird geprüft …', 'Checking the file …');
  static String get applying =>
      tr('Der Import wird angewendet …', 'Applying the import …');
  static String previewIntro(String fileName) => tr(
    'Datei: $fileName. Ein Import ersetzt alle Labels und Regionen der '
        'aufgeführten Aufnahmen. Erst „Anwenden“ ändert etwas.',
    'File: $fileName. An import replaces all labels and regions of the '
        'recordings listed. Nothing changes before “Apply”.',
  );
  static String get nothingReady => tr(
    'Keine Aufnahme kann übernommen werden.',
    'No recording can be taken over.',
  );
  static String get overwrite => tr(
    'Auch zwischenzeitlich geänderte Einträge überschreiben',
    'Also overwrite entries that were changed meanwhile',
  );
  static String get overwriteHint => tr(
    'Bei Konflikten gilt dann die Datei.',
    'With conflicts, the file then wins.',
  );
  static String get leftAsIsNote => tr(
    'Aufnahmen ohne Bereitschaft bleiben unverändert.',
    'Recordings that are not ready stay as they are.',
  );
  static String get unsupportedFile => tr(
    'Nur JSON- und CSV-Dateien lassen sich importieren.',
    'Only JSON and CSV files can be imported.',
  );
  static String importFailed(String reason) => tr(
    'Der Import ist fehlgeschlagen. $reason',
    'The import failed. $reason',
  );

  static String done(int applied, int leftAsIs) => leftAsIs == 0
      ? tr(
          applied == 1
              ? '1 Aufnahme wurde aktualisiert.'
              : '$applied Aufnahmen wurden aktualisiert.',
          applied == 1
              ? '1 recording was updated.'
              : '$applied recordings were updated.',
        )
      : tr(
          '$applied von ${applied + leftAsIs} Aufnahmen wurden aktualisiert, '
              '$leftAsIs blieben unverändert.',
          '$applied of ${applied + leftAsIs} recordings were updated, '
              '$leftAsIs stayed as they were.',
        );

  static String statusLabel(ImportStatus status) => switch (status) {
    ImportStatus.ready => tr('Bereit', 'Ready'),
    ImportStatus.conflict => tr('Konflikt', 'Conflict'),
    ImportStatus.invalid => tr('Ungültig', 'Invalid'),
    ImportStatus.unknownRecording => tr(
      'Unbekannte Aufnahme',
      'Unknown recording',
    ),
  };

  static String recordingFallback(int id) =>
      tr('Aufnahme $id', 'Recording $id');

  static String counts({
    required int added,
    required int changed,
    required int unchanged,
  }) => tr(
    'Neu $added · Geändert $changed · Unverändert $unchanged',
    'New $added · Changed $changed · Unchanged $unchanged',
  );

  static String removed(int count) => tr('Entfernt $count', 'Removed $count');
  static String removedWarning(int count) => tr(
    'Entfernt $count: Diese Bookmarks fehlen in der Datei und werden gelöscht.',
    'Removed $count: these bookmarks are not in the file and will be deleted.',
  );

  /// Where an issue lies, counted for people from 1.
  static String entry(ImportEntryKind kind, int index) => switch (kind) {
    ImportEntryKind.label => tr('Label ${index + 1}', 'Label ${index + 1}'),
    ImportEntryKind.region => tr('Region ${index + 1}', 'Region ${index + 1}'),
  };

  static String get originalGone => tr(
    'Die Markierung, aus der dieser Eintrag stammt, gibt es nicht mehr.',
    'The marker this entry was made from no longer exists.',
  );
  static String get originalChanged => tr(
    'Die Markierung, aus der dieser Eintrag stammt, wurde inzwischen '
        'geändert.',
    'The marker this entry was made from was changed in the meantime.',
  );
  static String get unknownRecording => tr(
    'Diese Aufnahme gibt es auf dem Server nicht.',
    'This recording does not exist on the server.',
  );
}
