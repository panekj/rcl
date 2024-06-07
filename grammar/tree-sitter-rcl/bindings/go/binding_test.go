package tree_sitter_rcl_test

import (
	"testing"

	tree_sitter "github.com/smacker/go-tree-sitter"
	"github.com/tree-sitter/tree-sitter-rcl"
)

func TestCanLoadGrammar(t *testing.T) {
	language := tree_sitter.NewLanguage(tree_sitter_rcl.Language())
	if language == nil {
		t.Errorf("Error loading Rcl grammar")
	}
}
