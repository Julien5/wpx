import 'dart:developer' as developer;
import 'dart:io';
import 'package:file_picker/file_picker.dart';
import 'package:file_saver/file_saver.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';

enum Type { pdf, gpx }

String fileExtension(Type type) {
  if (type == Type.pdf) {
    return "pdf";
  }
  return "gpx";
}

MimeType mimeType(Type type) {
  if (type == Type.pdf) {
    return MimeType.pdf;
  }
  return MimeType.custom;
}

FileType fileType(Type type) {
  if (type == Type.pdf) {
    return FileType.custom;
  }
  return FileType.custom;
}

void fileSave(List<int> data, Type type) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "waypoints",
      bytes: Uint8List.fromList(data),
      fileExtension: fileExtension(type),
      mimeType: mimeType(type),
    );
  } else if (Platform.isLinux) {
    var filepath = await FilePicker.platform.saveFile(
      fileName: "waypoints",
      type: fileType(type),
      allowedExtensions: [fileExtension(type)],
      bytes: Uint8List.fromList(data),
    );
    if (filepath == null) {
      return;
    }
    await Process.run('xdg-open', [filepath]);
  }
}

Future<List<int>> generate(SegmentsProvider provider, Type type) async {
  if (type == Type.pdf) {
    var data = await provider.generatePdf();
    return data;
  }
  assert(type == Type.gpx);
  //var data = await provider.generatePdf();
  List<int> data = [];
  return data;
}

class ExportButton extends StatefulWidget {
  final Type type;
  const ExportButton({super.key, required this.type});

  @override
  State<ExportButton> createState() => _ExportButtonState();
}

class _ExportButtonState extends State<ExportButton> {
  int length = 0;

  void onPressed(SegmentsProvider provider) async {
    if (!mounted) {
      return;
    }
    setState(() {
      length = 0;
    });
    var data = await generate(provider, widget.type);
    fileSave(data, widget.type);
    setState(() {
      developer.log("export length: ${data.length}");
      length = data.length;
    });
  }

  @override
  Widget build(BuildContext context) {
    SegmentsProvider model = Provider.of<SegmentsProvider>(context);
    return Row(
      children: [
        ElevatedButton(
          onPressed: () => onPressed(model),
          child: Text(fileExtension(widget.type)),
        ),
        SizedBox(width: 20,),
        Text("length: $length"),
      ],
    );
  }
}

class ExportWidget extends StatelessWidget {
  final SegmentsProvider segmentsProvider;

  const ExportWidget({super.key, required this.segmentsProvider});

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,

      children: [ExportButton(type: Type.pdf),SizedBox(height:20), ExportButton(type: Type.gpx)],
    );
  }
}

class ExportConsumer extends StatelessWidget {
  const ExportConsumer({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[ExportConsumer] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  Expanded(
                    child: ExportWidget(segmentsProvider: segmentsProvider),
                  ),
                ],
              ),
            ],
          ),
        );
      },
    );
  }
}

class ExportProviderWidget extends StatelessWidget {
  const ExportProviderWidget({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Export')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        if (rootModel.provider() == null) {
          return wait();
        }
        developer.log(
          "[SegmentsProviderWidget] ${rootModel.provider()?.filename()} length=${rootModel.provider()?.segments().length}",
        );
        return ChangeNotifierProvider.value(
          value: rootModel.provider(),
          builder: (context, child) {
            return Scaffold(
              appBar: AppBar(title: const Text('Export')),
              body: ExportConsumer(),
            );
          },
        );
      },
    );
  }
}
