import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/screens/home/filelist.dart';

class TrackFileListWidget extends StatefulWidget {
  final void Function(bridge.TrackFile) onTrackFileSelected;
  const TrackFileListWidget({super.key, required this.onTrackFileSelected});

  @override
  State<TrackFileListWidget> createState() => _TrackFileListWidgetState();
}

typedef TrackFileList = List<bridge.TrackFile>;

class _TrackFileListWidgetState extends State<TrackFileListWidget> {
  late Future<TrackFileList> _future;

  @override
  void initState() {
    super.initState();
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _future = context.read<RootModel>().trackFiles();
  }

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<TrackFileList>(
      future: _future,
      builder: (context, snapshot) {
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Text('waiting');
          //return const CircularProgressIndicator();
        }
        if (snapshot.hasError) {
          return Text('Error: ${snapshot.error}');
        }
        final filelist = snapshot.data!;
        if (filelist.isEmpty) {
          return const Text('no tracks saved');
        }
        return FileListWidget(
          files: filelist,
          onTrackFileSelected: widget.onTrackFileSelected,
        );
      },
    );
  }
}
