import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart';

/// A Material-styled, sortable list of GPX files.
///
/// Uses the standard [DataTable] widget so it inherits your app's
/// default Material theme (colors, text styles, elevation, etc.)
/// without any custom styling.
class FileListWidget extends StatefulWidget {
  final List<TrackFile> files;
  final void Function(TrackFile) onTrackFileSelected;
  final bool isLoading;
  const FileListWidget({
    super.key,
    required this.files,
    required this.onTrackFileSelected,
    required this.isLoading,
  });

  @override
  State<FileListWidget> createState() => _FileListWidgetState();
}

class _FileListWidgetState extends State<FileListWidget> {
  late List<TrackFile> _files;
  int? clickedIndex;

  @override
  void initState() {
    super.initState();
    _files = List.of(widget.files);
    _sort(0, ascending: true);
  }

  void _sort(int columnIndex, {required bool ascending}) {
    setState(() {
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

  String _formatDatetime(String datetimeData) {
    try {
      DateTime parsed = parseDateTime(datetimeData);
      return formatDate(parsed);
    } catch (e) {
      return "/";
    }
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

          Widget deleteWidget = SizedBox.shrink();

          if (widget.isLoading) {
            if (clickedIndex == index) {
              deleteWidget = const SizedBox(
                height: 15.0,
                width: 15.0,
                child: Center(child: CircularProgressIndicator()),
              );
            }
          } else {
            deleteWidget = IconButton(
              icon: const Icon(Icons.delete_outline, size: 20),
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints(minWidth: 20),
              visualDensity: VisualDensity.compact,
              tooltip: 'Delete',
              onPressed: () async => await _confirmDelete(backend, index),
            );
          }

          return DataRow(
            onSelectChanged:
                widget.isLoading
                    ? null
                    : (selected) {
                      if (selected != null && selected == true) {
                        clickedIndex = index;
                        widget.onTrackFileSelected(file);
                      }
                    },
            cells: [
              DataCell(
                ConstrainedBox(
                  constraints: const BoxConstraints(minWidth: 150),
                  child: Text(file.name),
                ),
              ),
              DataCell(Text(_formatLength(file.length))),
              DataCell(Text(_formatDatetime(file.startTime))),
              DataCell(deleteWidget),
            ],
          );
        }),
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
