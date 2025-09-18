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
    return FileType.any;
  }
  return FileType.any;
}

void fileSave(List<int> data, Type type) async {
  if (kIsWeb) {
    await FileSaver.instance.saveFile(
      name: "waypoints",
      bytes: Uint8List.fromList(data),
      fileExtension: fileExtension(type),
      mimeType: mimeType(type),
      customMimeType: fileExtension(type),
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

Future<List<int>> generate(RootModel root, Type type) async {
  // TODO
  List<int> L = List.empty();
  return L;
}

class ExportButton extends StatefulWidget {
  final Type type;
  const ExportButton({super.key, required this.type});

  @override
  State<ExportButton> createState() => _ExportButtonState();
}

class _ExportButtonState extends State<ExportButton> {
  int length = 0;

  void onPressed(RootModel root) async {
    if (!mounted) {
      return;
    }
    setState(() {
      length = 0;
    });
    var data = await generate(root, widget.type);
    fileSave(data, widget.type);
    setState(() {
      developer.log("export length: ${data.length}");
      length = data.length;
    });
  }

  @override
  Widget build(BuildContext context) {
    RootModel model = Provider.of<RootModel>(context);
    return Row(
      children: [
        ElevatedButton(
          onPressed: () => onPressed(model),
          child: Text(fileExtension(widget.type)),
        ),
        SizedBox(width: 20),
        Text("length: $length"),
      ],
    );
  }
}

class ExportWidget extends StatelessWidget {
  const ExportWidget({super.key});
  @override
  Widget build(BuildContext ctx) {
    Widget column = Column(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        ExportButton(type: Type.pdf),
        SizedBox(height: 20),
        ExportButton(type: Type.gpx),
      ],
    );
    return Center(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [Expanded(child: column)],
          ),
        ],
      ),
    );
  }
}

class ExportScaffold extends StatelessWidget {
  const ExportScaffold({super.key});
  @override
  Widget build(BuildContext ctx) {
    return Scaffold(
      appBar: AppBar(title: const Text('Export')),
      body: ExportWidget(),
    );
  }
}
