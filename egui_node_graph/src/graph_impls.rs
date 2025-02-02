use super::*;

impl<NodeData, DataType, ValueType> Graph<NodeData, DataType, ValueType> {
    pub fn new() -> Self {
        Self {
            nodes: SlotMap::default(),
            inputs: SlotMap::default(),
            outputs: SlotMap::default(),
            connections: SecondaryMap::default(),
        }
    }

    pub fn add_node(
        &mut self,
        label: String,
        user_data: NodeData,
        f: impl FnOnce(&mut Graph<NodeData, DataType, ValueType>, NodeId),
    ) -> NodeId {
        let node_id = self.nodes.insert_with_key(|node_id| {
            Node {
                id: node_id,
                label,
                // These get filled in later by the user function
                inputs: Vec::default(),
                outputs: Vec::default(),
                user_data,
            }
        });

        f(self, node_id);

        node_id
    }

    pub fn add_input_param(
        &mut self,
        node_id: NodeId,
        name: String,
        typ: DataType,
        value: ValueType,
        kind: InputParamKind,
        shown_inline: bool,
    ) -> InputId {
        let input_id = self.inputs.insert_with_key(|input_id| InputParam {
            id: input_id,
            typ,
            value,
            kind,
            node: node_id,
            shown_inline,
        });
        self.nodes[node_id].inputs.push((name, input_id));
        input_id
    }

    pub fn add_output_param(&mut self, node_id: NodeId, name: String, typ: DataType) -> OutputId {
        let output_id = self.outputs.insert_with_key(|output_id| OutputParam {
            id: output_id,
            node: node_id,
            typ,
        });
        self.nodes[node_id].outputs.push((name, output_id));
        output_id
    }

    pub fn remove_node(&mut self, node_id: NodeId) {
        self.connections
            .retain(|i, o| !(self.outputs[*o].node == node_id || self.inputs[i].node == node_id));
        let inputs: SVec<_> = self[node_id].input_ids().collect();
        for input in inputs {
            self.inputs.remove(input);
        }
        let outputs: SVec<_> = self[node_id].output_ids().collect();
        for output in outputs {
            self.outputs.remove(output);
        }
        self.nodes.remove(node_id);
    }

    pub fn remove_connection(&mut self, input_id: InputId) -> Option<OutputId> {
        self.connections.remove(input_id)
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.iter().map(|(id, _)| id)
    }

    pub fn add_connection(&mut self, output: OutputId, input: InputId) {
        self.connections.insert(input, output);
    }

    pub fn iter_connections(&self) -> impl Iterator<Item = (InputId, OutputId)> + '_ {
        self.connections.iter().map(|(o, i)| (o, *i))
    }

    pub fn connection(&self, input: InputId) -> Option<OutputId> {
        self.connections.get(input).copied()
    }

    pub fn any_param_type(&self, param: AnyParameterId) -> Result<&DataType, EguiGraphError> {
        match param {
            AnyParameterId::Input(input) => self.inputs.get(input).map(|x| &x.typ),
            AnyParameterId::Output(output) => self.outputs.get(output).map(|x| &x.typ),
        }
        .ok_or(EguiGraphError::InvalidParameterId(param))
    }

    pub fn get_input(&self, input: InputId) -> &InputParam<DataType, ValueType> {
        &self.inputs[input]
    }

    pub fn get_output(&self, output: OutputId) -> &OutputParam<DataType> {
        &self.outputs[output]
    }
}

impl<NodeData, DataType, ValueType> Default for Graph<NodeData, DataType, ValueType> {
    fn default() -> Self {
        Self::new()
    }
}

impl<NodeData> Node<NodeData> {
    pub fn inputs<'a, DataType, DataValue>(
        &'a self,
        graph: &'a Graph<NodeData, DataType, DataValue>,
    ) -> impl Iterator<Item = &InputParam<DataType, DataValue>> + 'a {
        self.input_ids().map(|id| graph.get_input(id))
    }

    pub fn outputs<'a, DataType, DataValue>(
        &'a self,
        graph: &'a Graph<NodeData, DataType, DataValue>,
    ) -> impl Iterator<Item = &OutputParam<DataType>> + 'a {
        self.output_ids().map(|id| graph.get_output(id))
    }

    pub fn input_ids(&self) -> impl Iterator<Item = InputId> + '_ {
        self.inputs.iter().map(|(_name, id)| *id)
    }

    pub fn output_ids(&self) -> impl Iterator<Item = OutputId> + '_ {
        self.outputs.iter().map(|(_name, id)| *id)
    }

    pub fn get_input(&self, name: &str) -> Result<InputId, EguiGraphError> {
        self.inputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }

    pub fn get_output(&self, name: &str) -> Result<OutputId, EguiGraphError> {
        self.outputs
            .iter()
            .find(|(param_name, _id)| param_name == name)
            .map(|x| x.1)
            .ok_or_else(|| EguiGraphError::NoParameterNamed(self.id, name.into()))
    }
}

impl<DataType, ValueType> InputParam<DataType, ValueType> {
    pub fn value(&self) -> &ValueType {
        &self.value
    }

    pub fn kind(&self) -> InputParamKind {
        self.kind
    }

    pub fn node(&self) -> NodeId {
        self.node
    }
}
