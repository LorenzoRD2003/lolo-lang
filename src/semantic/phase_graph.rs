// Representa el DAG de dependencias entre fases
// Nota: el diseño mas escalable posible seria
// in_degree: HashMap<&str, usize>
// adjacency: HashMap<&str, Vec<&str>>
// Pero es alto overkill para mis pocas fases de analisis semantico

use std::collections::HashSet;

use crate::semantic::phase::{
  CategoryCheckerPhase, CompileTimeConstantCheckerPhase, MutabilityCheckerPhase, NameResolverPhase,
  SemanticPhase, TypeCheckerPhase,
};

pub struct PhaseNode<'a> {
  pub name: &'static str,
  pub phase: Box<dyn SemanticPhase<'a>>,
  pub dependencies: Vec<&'static str>,
}

impl<'a> PhaseNode<'a> {
  pub fn new(
    name: &'static str,
    phase: Box<dyn SemanticPhase<'a>>,
    dependencies: Vec<&'static str>,
  ) -> Self {
    Self {
      name,
      phase,
      dependencies,
    }
  }
}

pub struct PhaseGraph<'a> {
  nodes: Vec<PhaseNode<'a>>,
  completed_phases: HashSet<&'static str>,
}

impl<'a> PhaseGraph<'a> {
  pub fn from(nodes: Vec<PhaseNode<'a>>) -> Self {
    Self {
      nodes,
      completed_phases: HashSet::new(),
    }
  }

  /// Indica cuales fases estan listas para ser ejecutadas, pero aun no fueron completadas.
  /// Por ejemplo, si `completed = { NameResolver }`, entonces devuelve `[TypeChecker, MutabilityChecker]`
  pub fn ready_phases(&self) -> Vec<&PhaseNode<'a>> {
    // es cuadratico pero no me importa. son pocos nodos. hacer un orden topologico aca es mucho laburo.
    self
      .nodes
      .iter()
      .filter(|node| {
        !self.completed_phases.contains(node.name)
          && node
            .dependencies
            .iter()
            .all(|dep| self.completed_phases.contains(dep))
      })
      .collect()
  }

  /// Registra que una fase termino correctamente.
  /// Entonces, en la siguiente ejecucion del iterador, `ready_phases` puede detectar nuevas fases ejecutables.
  pub fn mark_phase_completed(&mut self, name: &'static str) {
    self.completed_phases.insert(name);
  }

  /// Indica si todas las fases terminaron correctamente.
  pub fn all_phases_completed(&self) -> bool {
    self.nodes.len() == self.completed_phases.len()
  }

  // Grafo de dependencias asociado al compilador
  // (fase1)
  // NameResolver
  // CompileTimeConstantChecker
  // (fase2)
  // NameResolver → TypeChecker
  // NameResolver → MutabilityChecker
  // CompileTimeConstantChecker → CategoryChecker
  pub fn default_semantic_graph() -> Self {
    PhaseGraph::from(vec![
      PhaseNode::new("NameResolver", Box::new(NameResolverPhase), vec![]),
      PhaseNode::new(
        "CompileTimeConstantChecker",
        Box::new(CompileTimeConstantCheckerPhase),
        vec![],
      ),
      PhaseNode::new(
        "TypeChecker",
        Box::new(TypeCheckerPhase),
        vec!["NameResolver"],
      ),
      PhaseNode::new(
        "MutabilityChecker",
        Box::new(MutabilityCheckerPhase),
        vec!["NameResolver"],
      ),
      PhaseNode::new(
        "CategoryChecker",
        Box::new(CategoryCheckerPhase),
        vec!["CompileTimeConstantChecker"],
      ),
    ])
  }
}
