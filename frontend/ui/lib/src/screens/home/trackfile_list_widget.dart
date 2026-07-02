import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

class TrackFileListWidget extends StatefulWidget {
  const TrackFileListWidget({super.key});

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
        final data = snapshot.data!;
        if (data.isEmpty) {
          return const Text('no tracks saved');
        }
        return ListView.builder(
          itemCount: data.length,
          itemBuilder: (context, index) {
            return Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Text(data[index].name),
            );
          },
        );
      },
    );
  }
}
