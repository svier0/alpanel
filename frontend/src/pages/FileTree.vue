<template>
  <div class="tree" :class="{ loading }">
    <template v-for="node in nodes" :key="node.path">
      <div
        class="tree-node"
        :class="{ active: activePath === node.path && !node.is_dir }"
        :style="{ paddingLeft: (depth ?? 0) * 14 + 8 + 'px' }"
        @click="onClick(node)"
      >
        <span class="tree-arrow" v-if="node.is_dir">
          <el-icon v-if="node.expanded"><ArrowDown /></el-icon>
          <el-icon v-else><ArrowRight /></el-icon>
        </span>
        <span class="tree-arrow placeholder" v-else />
        <el-icon v-if="node.is_dir" class="tree-icon"><Folder /></el-icon>
        <el-icon v-else class="tree-icon"><Document /></el-icon>
        <span class="tree-label">{{ node.name }}</span>
      </div>
      <div v-if="node.is_dir && node.expanded && node.children?.length" class="tree-children">
        <FileTree
          :nodes="node.children"
          :depth="(depth ?? 0) + 1"
          :loading="loading"
          :active-path="activePath"
          :open-paths="openPaths"
          @toggle="(n: TreeNode) => emit('toggle', n)"
          @select="(n: TreeNode) => emit('select', n)"
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
import { ArrowDown, ArrowRight, Folder, Document } from '@element-plus/icons-vue'

export interface TreeNode {
  name: string
  path: string
  is_dir: boolean
  loaded: boolean
  expanded: boolean
  children?: TreeNode[]
}

defineProps<{
  nodes: TreeNode[]
  depth?: number
  loading?: boolean
  activePath?: string
  openPaths?: Set<string>
}>()

const emit = defineEmits<{
  toggle: [TreeNode]
  select: [TreeNode]
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

.tree-empty {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  height: 24px;
  display: flex;
  align-items: center;
}
</style>
