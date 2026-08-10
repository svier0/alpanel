<template>
  <div class="tree" :class="{ loading }">
    <template v-for="node in nodes" :key="node.path">
      <div
        class="tree-node"
        :class="{ active: activePath === node.path && !node.is_dir, renaming: renamingPath === node.path }"
        :style="{ paddingLeft: (depth ?? 0) * 14 + 8 + 'px' }"
        @click="onClick(node)"
        @contextmenu.prevent="(e: MouseEvent) => emit('ctx', node, e)"
      >
        <span class="tree-arrow" v-if="node.is_dir">
          <el-icon v-if="node.expanded"><ArrowDown /></el-icon>
          <el-icon v-else><ArrowRight /></el-icon>
        </span>
        <span class="tree-arrow placeholder" v-else />
        <el-icon v-if="node.is_dir" class="tree-icon"><Folder /></el-icon>
        <el-icon v-else class="tree-icon"><Document /></el-icon>
        <span
          v-if="renamingPath !== node.path"
          class="tree-label"
        >{{ node.name }}</span>
        <span v-else class="tree-rename" @click.stop>
          <el-input
            v-model="renamingValue"
            size="small"
            class="tree-rename-input"
            :suffix-icon="Check"
            @keyup.enter="emit('rename', node)"
            @keyup.escape="emit('cancelRename')"
            @blur="emit('rename', node)"
          />
        </span>
      </div>
      <div v-if="node.is_dir && node.expanded && node.children?.length" class="tree-children">
        <FileTree
          :nodes="node.children"
          :depth="(depth ?? 0) + 1"
          :loading="loading"
          :active-path="activePath"
          :open-paths="openPaths"
          :renaming-path="renamingPath"
          :renaming-value="renamingValue"
          @toggle="(n: TreeNode) => emit('toggle', n)"
          @select="(n: TreeNode) => emit('select', n)"
          @ctx="(n: TreeNode, e: MouseEvent) => emit('ctx', n, e)"
          @rename="(n: TreeNode) => emit('rename', n)"
          @cancel-rename="emit('cancelRename')"
        />
      </div>
      <div v-else-if="node.is_dir && node.expanded && !node.children?.length" class="tree-empty" :style="{ paddingLeft: ((depth ?? 0) + 1) * 14 + 24 + 'px' }">
        空目录
      </div>
    </template>
    <div v-if="!nodes.length && !loading" class="tree-empty">无内容</div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { ArrowDown, ArrowRight, Folder, Document, Check } from '@element-plus/icons-vue'

export interface TreeNode {
  name: string
  path: string
  is_dir: boolean
  loaded: boolean
  expanded: boolean
  children?: TreeNode[]
}

const props = defineProps<{
  nodes: TreeNode[]
  depth?: number
  loading?: boolean
  activePath?: string
  openPaths?: Set<string>
  renamingPath?: string
  renamingValue?: string
}>()

const renamingValue = ref(props.renamingValue || '')

watch(() => props.renamingValue, (v) => {
  if (v !== undefined) renamingValue.value = v
})

watch(() => props.renamingPath, (p) => {
  if (p) {
    nextTick(() => {
      const el = document.querySelector<HTMLInputElement>('.tree-rename-input input')
      if (el) {
        el.focus()
        el.select()
      }
    })
  }
})

const emit = defineEmits<{
  toggle: [TreeNode]
  select: [TreeNode]
  ctx: [TreeNode, MouseEvent]
  rename: [TreeNode]
  cancelRename: []
}>()

function onClick(node: TreeNode) {
  if (node.is_dir) emit('toggle', node)
  else emit('select', node)
}
</script>

<style scoped>
.tree { font-size: 12px; user-select: none; }

.tree-node {
  display: flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding-right: 8px;
  cursor: pointer;
  color: var(--el-text-color-regular);
  white-space: nowrap;
}
.tree-node:hover { background: var(--el-fill-color-light); }
.tree-node.active {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
  font-weight: 500;
}

.tree-arrow {
  width: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--el-text-color-secondary);
}
.tree-arrow.placeholder { visibility: hidden; }

.tree-icon { flex-shrink: 0; color: var(--el-text-color-secondary); }
.tree-label { overflow: hidden; text-overflow: ellipsis; }

.tree-node.renaming { background: var(--el-fill-color-light); }
.tree-rename { flex: 1; min-width: 0; margin-right: 8px; }
.tree-rename-input :deep(.el-input__wrapper) {
  padding-right: 2px;
}
.tree-rename-input :deep(.el-input__suffix) {
  cursor: pointer;
}
.tree-rename-input :deep(.el-input__suffix-inner) {
  color: var(--el-color-primary);
}

.tree-empty {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  height: 24px;
  display: flex;
  align-items: center;
}
</style>
