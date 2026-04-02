import { useEffect, useMemo, useState } from 'react'
import { Pencil, Plus, RefreshCw, Trash2 } from 'lucide-react'
import { toast } from 'sonner'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import {
  useAddCustomModel,
  useDeleteCustomModel,
  useModels,
  useUpdateCustomModel,
} from '@/hooks/use-credentials'
import { extractErrorMessage } from '@/lib/utils'
import type { AddCustomModelRequest, CustomModelItem } from '@/types/api'

interface ModelManagementProps {
  open: boolean
  onOpenChange: (open: boolean) => void
}

const emptyForm: AddCustomModelRequest = {
  id: '',
  displayName: '',
  modelType: 'chat',
  maxTokens: 64000,
  ownedBy: 'custom',
  contextWindow: 1_000_000,
  supportsThinking: true,
  credentialTier: 'any',
}

export function ModelManagementDialog({ open, onOpenChange }: ModelManagementProps) {
  const [editingModel, setEditingModel] = useState<CustomModelItem | null>(null)
  const [form, setForm] = useState<AddCustomModelRequest>(emptyForm)

  const { data, isLoading, error, refetch, isFetching } = useModels()
  const { mutate: addCustomModel, isPending: isAdding } = useAddCustomModel()
  const { mutate: updateCustomModel, isPending: isUpdating } = useUpdateCustomModel()
  const { mutate: deleteCustomModel, isPending: isDeleting } = useDeleteCustomModel()

  const submitting = isAdding || isUpdating

  const builtInOptions = useMemo(() => data?.builtIn ?? [], [data?.builtIn])
  const customModels = useMemo(() => data?.custom ?? [], [data?.custom])

  useEffect(() => {
    if (!open) {
      setEditingModel(null)
      setForm(emptyForm)
    }
  }, [open])

  useEffect(() => {
    if (!editingModel) {
      return
    }

    setForm({
      id: editingModel.id,
      displayName: editingModel.displayName,
      modelType: editingModel.modelType,
      maxTokens: editingModel.maxTokens,
      ownedBy: editingModel.ownedBy,
      contextWindow: editingModel.contextWindow,
      supportsThinking: editingModel.supportsThinking,
      credentialTier: editingModel.credentialTier,
    })
  }, [editingModel])

  const resetForm = () => {
    setEditingModel(null)
    setForm(emptyForm)
  }

  const handleChange = <K extends keyof AddCustomModelRequest>(key: K, value: AddCustomModelRequest[K]) => {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  const handleEdit = (model: CustomModelItem) => {
    setEditingModel(model)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    if (!form.id.trim()) {
      toast.error('请输入真实上游模型 ID')
      return
    }
    if (!form.displayName.trim()) {
      toast.error('请输入显示名称')
      return
    }
    if (!Number.isFinite(form.maxTokens) || form.maxTokens <= 0) {
      toast.error('最大 Token 必须大于 0')
      return
    }
    if (!Number.isFinite(form.contextWindow) || form.contextWindow <= 0) {
      toast.error('上下文窗口必须大于 0')
      return
    }

    const payload: AddCustomModelRequest = {
      id: form.id.trim(),
      displayName: form.displayName.trim(),
      modelType: form.modelType.trim() || 'chat',
      maxTokens: form.maxTokens,
      ownedBy: form.ownedBy.trim() || 'custom',
      contextWindow: form.contextWindow,
      supportsThinking: form.supportsThinking,
      credentialTier: form.credentialTier,
    }

    if (editingModel) {
      updateCustomModel(
        { id: editingModel.id, req: payload },
        {
          onSuccess: (response) => {
            toast.success(response.message)
            resetForm()
          },
          onError: (err: unknown) => {
            toast.error(`更新失败: ${extractErrorMessage(err)}`)
          },
        }
      )
      return
    }

    addCustomModel(payload, {
      onSuccess: (response) => {
        toast.success(response.message)
        resetForm()
      },
      onError: (err: unknown) => {
        toast.error(`添加失败: ${extractErrorMessage(err)}`)
      },
    })
  }

  const handleDelete = (id: string) => {
    if (!confirm(`确定要删除自定义模型 ${id} 吗？此操作无法撤销。`)) {
      return
    }

    deleteCustomModel(id, {
      onSuccess: (response) => {
        toast.success(response.message)
        if (editingModel?.id === id) {
          resetForm()
        }
      },
      onError: (err: unknown) => {
        toast.error(`删除失败: ${extractErrorMessage(err)}`)
      },
    })
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-6xl max-h-[90vh] overflow-hidden">
        <DialogHeader>
          <DialogTitle>模型管理</DialogTitle>
        </DialogHeader>

        <div className="grid gap-6 lg:grid-cols-[1.1fr_1.4fr]">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">
                {editingModel ? `编辑模型 ${editingModel.id}` : '添加自定义模型'}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <form onSubmit={handleSubmit} className="space-y-4">
                <div className="space-y-2">
                  <label className="text-sm font-medium">真实上游模型 ID</label>
                  <Input
                    value={form.id}
                    onChange={(e) => handleChange('id', e.target.value)}
                    placeholder="例如：claude-sonnet-4.7"
                    disabled={submitting}
                  />
                </div>

                <div className="space-y-2">
                  <label className="text-sm font-medium">显示名称</label>
                  <Input
                    value={form.displayName}
                    onChange={(e) => handleChange('displayName', e.target.value)}
                    placeholder="例如：Claude Sonnet 4.7"
                    disabled={submitting}
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">模型类型</label>
                    <Input
                      value={form.modelType}
                      onChange={(e) => handleChange('modelType', e.target.value)}
                      placeholder="chat"
                      disabled={submitting}
                    />
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm font-medium">所属方</label>
                    <Input
                      value={form.ownedBy}
                      onChange={(e) => handleChange('ownedBy', e.target.value)}
                      placeholder="custom"
                      disabled={submitting}
                    />
                  </div>
                </div>

                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">最大 Token</label>
                    <Input
                      type="number"
                      min="1"
                      value={form.maxTokens}
                      onChange={(e) => handleChange('maxTokens', Number(e.target.value) || 0)}
                      disabled={submitting}
                    />
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm font-medium">上下文窗口</label>
                    <Input
                      type="number"
                      min="1"
                      value={form.contextWindow}
                      onChange={(e) => handleChange('contextWindow', Number(e.target.value) || 0)}
                      disabled={submitting}
                    />
                  </div>
                </div>

                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">支持 Thinking</label>
                    <select
                      value={form.supportsThinking ? 'true' : 'false'}
                      onChange={(e) => handleChange('supportsThinking', e.target.value === 'true')}
                      disabled={submitting}
                      className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <option value="true">是</option>
                      <option value="false">否</option>
                    </select>
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm font-medium">凭据等级</label>
                    <select
                      value={form.credentialTier}
                      onChange={(e) => handleChange('credentialTier', e.target.value as AddCustomModelRequest['credentialTier'])}
                      disabled={submitting}
                      className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50"
                    >
                      <option value="any">any</option>
                      <option value="opus">opus</option>
                    </select>
                  </div>
                </div>

                <div className="rounded-md border p-3 text-sm text-muted-foreground">
                  保存后，模型 ID 会直接作为最终发给上游的真实 `modelId`，不再映射到其它内置模型。
                </div>

                <DialogFooter className="pt-2">
                  {editingModel && (
                    <Button type="button" variant="outline" onClick={resetForm} disabled={submitting}>
                      取消编辑
                    </Button>
                  )}
                  <Button type="submit" disabled={submitting}>
                    {submitting ? '提交中...' : editingModel ? '保存修改' : '添加模型'}
                  </Button>
                </DialogFooter>
              </form>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0">
              <CardTitle className="text-lg">自定义模型列表</CardTitle>
              <Button variant="outline" size="sm" onClick={() => refetch()} disabled={isFetching}>
                <RefreshCw className={`h-4 w-4 mr-2 ${isFetching ? 'animate-spin' : ''}`} />
                刷新
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="rounded-md border p-3 text-sm text-muted-foreground">
                当前内置模型 {builtInOptions.length} 个，自定义模型 {customModels.length} 个。
              </div>

              {isLoading ? (
                <div className="py-8 text-center text-muted-foreground">加载中...</div>
              ) : error ? (
                <div className="py-8 text-center text-destructive">
                  加载失败：{extractErrorMessage(error)}
                </div>
              ) : customModels.length === 0 ? (
                <div className="py-8 text-center text-muted-foreground">暂无自定义模型</div>
              ) : (
                <div className="space-y-3 max-h-[56vh] overflow-y-auto pr-1">
                  {customModels.map((model) => (
                    <div key={model.id} className="rounded-lg border p-4 space-y-3">
                      <div className="flex items-start justify-between gap-4">
                        <div>
                          <div className="font-medium">{model.displayName}</div>
                          <div className="text-sm text-muted-foreground">{model.id}</div>
                        </div>
                        <div className="flex items-center gap-2">
                          <Button type="button" size="sm" variant="outline" onClick={() => handleEdit(model)}>
                            <Pencil className="h-4 w-4 mr-2" />
                            编辑
                          </Button>
                          <Button
                            type="button"
                            size="sm"
                            variant="destructive"
                            onClick={() => handleDelete(model.id)}
                            disabled={isDeleting}
                          >
                            <Trash2 className="h-4 w-4 mr-2" />
                            删除
                          </Button>
                        </div>
                      </div>

                      <div className="grid gap-2 text-sm text-muted-foreground sm:grid-cols-2">
                        <div>上游模型 ID：{model.upstreamModelId ?? model.id}</div>
                        <div>模型类型：{model.modelType}</div>
                        <div>所属方：{model.ownedBy}</div>
                        <div>最大 Token：{model.maxTokens}</div>
                        <div>上下文窗口：{model.contextWindow}</div>
                        <div>支持 Thinking：{model.supportsThinking ? '是' : '否'}</div>
                        <div>凭据等级：{model.credentialTier}</div>
                      </div>
                    </div>
                  ))}
                </div>
              )}

              <div className="rounded-md border p-3">
                <div className="mb-2 flex items-center gap-2 text-sm font-medium">
                  <Plus className="h-4 w-4" />
                  当前内置模型参考
                </div>
                <div className="max-h-48 space-y-2 overflow-y-auto pr-1 text-sm text-muted-foreground">
                  {builtInOptions.map((model) => (
                    <div key={model.id} className="rounded border px-3 py-2">
                      {model.displayName} ({model.id})
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      </DialogContent>
    </Dialog>
  )
}
