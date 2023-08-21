# THIS IS WIP

# Surfaceflinger Hook

* 通过ptrace注入so，并且inline hook surfaceflinger以获取frametime数据

# Build

```bash
# clone
git clone https://github.com/shadow3aaa/surfacefliger_hook.git
cd surfacefliger_hook

# sync submodule
git submodule init
git submodule update

# build
make -j4
```

# To do

* 做一个安全通讯jank的api
