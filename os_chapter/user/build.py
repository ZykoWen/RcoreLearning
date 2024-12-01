import os;

#每个应用的起始地址将从此基础值加偏移量计算
base_address = 0x80400000
#每个应用的内存地址步长
step = 0x20000
#链接器脚本的路径，存储了应用的内存分布信息
linker = 'src/linker.ld'

app_id = 0
#获取目录 src/bin 中的所有文件
apps = os.listdir('src/bin')
#对文件名进行字典序排序，确保每次运行脚本时，应用程序的构建顺序一致
apps.sort()

for app in apps:
  #去掉文件扩展名，提取纯应用程序名
  app = app[:app.find('.')]
  lines = []
  lines_before = []
  with open(linker, 'r') as f:
    for line in f.readlines():
      lines_before.append(line)
      #将脚本中的 base_address 替换为当前应用的起始地址
      line = line.replace(hex(base_address), hex(base_address + step * app_id))
      #保存修改后的脚本内容
      lines.append(line)
  #将修改后的内容写回链接器脚本
  with open(linker, 'w+') as f:
    f.writelines(lines)
  #cargo 构建当前应用
  os.system('cargo build --bin %s --release' % app)
  print('[build.py] application %s start with address %s' %(app, hex(base_address + step * app_id)))
  with open(linker, 'w+') as f:
    f.writelines(lines_before)
  app_id = app_id + 1


