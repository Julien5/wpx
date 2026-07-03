import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart'; // add `intl` to pubspec.yaml

/// A Material-styled, sortable list of GPX files.
///
/// Uses the standard [DataTable] widget so it inherits your app's
/// default Material theme (colors, text styles, elevation, etc.)
/// without any custom styling.
class FileListWidget extends StatefulWidget {
  final List<TrackFile> files;
  final void Function(TrackFile) onTrackFileSelected;
  const FileListWidget({
    super.key,
    required this.files,
    required this.onTrackFileSelected,
  });

  @override
  State<FileListWidget> createState() => _FileListWidgetState();
}

class _FileListWidgetState extends State<FileListWidget> {
  int _sortColumnIndex = 0;
  bool _sortAscending = true;
  late List<TrackFile> _files;

  @override
  void initState() {
    super.initState();
    _files = List.of(widget.files);
    _sort(0, ascending: true);
  }

  void _sort(int columnIndex, {required bool ascending}) {
    setState(() {
      _sortColumnIndex = columnIndex;
      _sortAscending = ascending;

      switch (columnIndex) {
        case 0:
          _files.sort(
            (a, b) =>
                ascending ? a.name.compareTo(b.name) : b.name.compareTo(a.name),
          );
          break;
        case 1:
          _files.sort(
            (a, b) =>
                ascending
                    ? a.length.compareTo(b.length)
                    : b.length.compareTo(a.length),
          );
          break;
        case 2:
          _files.sort(
            (a, b) =>
                ascending
                    ? a.length.compareTo(b.length)
                    : b.length.compareTo(a.length),
          );
          break;
      }
    });
  }

  Future<void> _renameFile(Bridge backend, int index, String newName) async {
    _files[index] = await backend.updateTrackfileName(
      trackfile: _files[index],
      name: newName,
    );
    setState(() {});
  }

  Future<void> _confirmDelete(Bridge backend, int index) async {
    final file = _files[index];
    final confirmed = await showDialog<bool>(
      context: context,
      builder:
          (context) => AlertDialog(
            title: const Text('Delete file'),
            content: Text('Delete "${file.name}"? This can\'t be undone.'),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: const Text('Cancel'),
              ),
              TextButton(
                onPressed: () => Navigator.pop(context, true),
                style: TextButton.styleFrom(
                  foregroundColor: Theme.of(context).colorScheme.error,
                ),
                child: const Text('Delete'),
              ),
            ],
          ),
    );

    if (confirmed == true) {
      await backend.removeTrackfile(trackfile: _files[index]);
      setState(() => _files.removeAt(index));
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Deleted "${file.name}"')));
      }
    }
  }

  String _formatLength(double meters) {
    if (meters < 1000) return '${meters.toStringAsFixed(0)} m';
    return '${(meters / 1000).toStringAsFixed(1)} km';
  }

  @override
  Widget build(BuildContext context) {
    Bridge backend = getBackend(context);
    return SingleChildScrollView(
      scrollDirection: Axis.vertical,
      child: DataTable(
        headingRowHeight: 0,
        columnSpacing: 8,
        horizontalMargin: 0,
        sortColumnIndex: null,
        showCheckboxColumn: false,
        columns: [
          DataColumn(
            label: const Text('Name'),
            onSort: (i, asc) => _sort(i, ascending: asc),
          ),
          DataColumn(
            label: const Text('Length'),
            numeric: true,
            onSort: (i, asc) => _sort(i, ascending: asc),
          ),
          DataColumn(
            label: const Text('Start date'),
            onSort: (i, asc) => _sort(i, ascending: asc),
          ),
          const DataColumn(label: SizedBox.shrink()),
        ],
        rows: List.generate(_files.length, (index) {
          final file = _files[index];
          return DataRow(
            onSelectChanged: (selected) {
              if (selected != null && selected == true) {
                widget.onTrackFileSelected(file);
              }
            },
            cells: [
              DataCell(
                ConstrainedBox(
                  constraints: const BoxConstraints(minWidth: 150),
                  child: _EditableNameCell(
                    key: ValueKey(file.hashCode),
                    initialName: file.name,
                    onSubmitted: (newName) async {
                      await _renameFile(backend, index, newName);
                    },
                  ),
                ),
              ),
              DataCell(Text(_formatLength(file.length))),
              DataCell(Text("TODO")),
              DataCell(
                IconButton(
                  icon: const Icon(Icons.delete_outline, size: 20),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(minWidth: 20),
                  visualDensity: VisualDensity.compact,
                  tooltip: 'Delete',
                  onPressed: () async => await _confirmDelete(backend, index),
                ),
              ),
            ],
          );
        }),
      ),
    );
  }
}

/// A table cell that displays a file name as plain text with a subtle
/// pencil icon, and turns into an inline, auto-selected [TextField] when
/// tapped. Commits on submit (Enter) or on losing focus; Escape cancels.
class _EditableNameCell extends StatefulWidget {
  final String initialName;
  final ValueChanged<String> onSubmitted;

  const _EditableNameCell({
    super.key,
    required this.initialName,
    required this.onSubmitted,
  });

  @override
  State<_EditableNameCell> createState() => _EditableNameCellState();
}

class _EditableNameCellState extends State<_EditableNameCell> {
  late final TextEditingController _controller = TextEditingController(
    text: widget.initialName,
  );
  late final FocusNode _focusNode =
      FocusNode()..addListener(() {
        if (!_focusNode.hasFocus && _isEditing) _commit();
      });
  bool _isEditing = false;
  bool _isHovering = false;

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    super.dispose();
  }

  void _startEditing() {
    setState(() => _isEditing = true);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
      _controller.selection = TextSelection(
        baseOffset: 0,
        extentOffset: _controller.text.length,
      );
    });
  }

  void _commit() {
    final newName = _controller.text.trim();
    setState(() => _isEditing = false);
    if (newName.isNotEmpty && newName != widget.initialName) {
      widget.onSubmitted(newName);
    } else {
      _controller.text = widget.initialName;
    }
  }

  void _cancel() {
    _controller.text = widget.initialName;
    setState(() => _isEditing = false);
    _focusNode.unfocus();
  }

  @override
  Widget build(BuildContext context) {
    if (_isEditing) {
      return SizedBox(
        width: 200,
        child: KeyboardListener(
          focusNode: FocusNode(),
          onKeyEvent: (event) {
            if (event is KeyDownEvent &&
                event.logicalKey == LogicalKeyboardKey.escape) {
              _cancel();
            }
          },
          child: TextField(
            controller: _controller,
            focusNode: _focusNode,
            style: Theme.of(context).textTheme.bodyMedium,
            decoration: const InputDecoration(
              isDense: true,
              contentPadding: EdgeInsets.symmetric(vertical: 4),
              border: UnderlineInputBorder(),
            ),
            onSubmitted: (_) => _commit(),
          ),
        ),
      );
    }

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: _startEditing,
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(widget.initialName),
            const SizedBox(width: 6),
            Opacity(
              opacity: _isHovering ? 1 : 0.4,
              child: const Icon(Icons.edit, size: 14),
            ),
          ],
        ),
      ),
    );
  }
}

/// --- Example usage / sample data ---
///
/// void main() => runApp(MaterialApp(
///       home: GpxFileListPage(
///         files: [
///           GpxFile(
///             name: 'Morning Ride',
///             lengthMeters: 21500,
///             startDate: DateTime(2026, 6, 12, 7, 30),
///           ),
///           GpxFile(
///             name: 'Alps Trek',
///             lengthMeters: 84200,
///             startDate: DateTime(2026, 5, 2, 6, 0),
///           ),
///         ],
///       ),
///     ));
